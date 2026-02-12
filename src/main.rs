mod analysis;
mod config;
mod error;
mod http;
mod llm;
mod output;
mod sources;
mod types;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(
    name = "st-narrative",
    about = "Solana narrative detection and idea generation tool"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Run the full narrative detection pipeline and generate a report
    Run {
        /// Path to config file
        #[arg(short, long, default_value = "config.toml")]
        config: PathBuf,

        /// Output path for the HTML report
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// LLM provider override: anthropic, openrouter, openai
        #[arg(long)]
        provider: Option<String>,

        /// LLM model override
        #[arg(long)]
        model: Option<String>,
    },

    /// Collect signals only (no Claude analysis), output as JSON
    Signals {
        /// Path to config file
        #[arg(short, long, default_value = "config.toml")]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "st_narrative=info".parse().unwrap()),
        )
        .init();

    dotenvy::from_path("../.env").ok();
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            config,
            output,
            provider,
            model,
        } => run(config, output, provider, model).await,
        Command::Signals { config } => signals_only(config).await,
    }
}

async fn run(
    config_path: PathBuf,
    output_override: Option<PathBuf>,
    provider_override: Option<String>,
    model_override: Option<String>,
) -> Result<()> {
    let mut cfg = config::Config::load(&config_path)
        .context(format!("loading config from {}", config_path.display()))?;
    cfg.validate()?;

    // Apply CLI overrides
    if let Some(p) = provider_override {
        cfg.llm.provider = match p.as_str() {
            "anthropic" => llm::Provider::Anthropic,
            "openai" => llm::Provider::OpenAi,
            _ => llm::Provider::OpenRouter,
        };
    }
    if let Some(m) = model_override {
        cfg.llm.model = m;
    }

    let output_path = output_override.unwrap_or_else(|| PathBuf::from(&cfg.output.path));
    let http_client = http::HttpClient::new("st-narrative/0.1.0 (solscout)")?;

    // Collect signals from all sources in parallel
    info!("collecting signals from all sources...");
    let (github_result, solana_result, social_result) = tokio::join!(
        sources::github::collect(&cfg.github, &http_client),
        sources::solana_rpc::collect(&cfg.solana, &http_client),
        sources::social::collect(&cfg.social, &http_client),
    );

    let mut signals = Vec::new();

    match github_result {
        Ok(s) => {
            info!(count = s.len(), "GitHub signals collected");
            signals.extend(s);
        }
        Err(e) => tracing::error!("GitHub collection failed: {e}"),
    }

    match solana_result {
        Ok(s) => {
            info!(count = s.len(), "Solana onchain signals collected");
            signals.extend(s);
        }
        Err(e) => tracing::error!("Solana RPC collection failed: {e}"),
    }

    match social_result {
        Ok(s) => {
            info!(count = s.len(), "Social signals collected");
            signals.extend(s);
        }
        Err(e) => tracing::error!("Social collection failed: {e}"),
    }

    if signals.is_empty() {
        anyhow::bail!(
            "No signals collected from any source. Check API keys and network connectivity."
        );
    }

    info!(total = signals.len(), "total signals collected");

    // Aggregate signals
    let groups = analysis::aggregator::aggregate(&signals);
    let signals_json = analysis::aggregator::signals_to_json(&signals, &groups);

    info!(groups = groups.len(), "signal groups formed");

    // LLM analysis: identify narratives
    let llm_client = llm::LlmClient::from_config(
        cfg.llm.provider.clone(),
        cfg.llm.model.clone(),
        cfg.llm.max_tokens,
        cfg.llm.api_key_env.clone(),
        cfg.llm.base_url.clone(),
    )?;

    let narratives = analysis::synthesizer::identify_narratives(&llm_client, &signals_json).await?;
    info!(count = narratives.len(), "narratives identified");

    // LLM analysis: generate build ideas
    let build_ideas = analysis::ideas::generate_ideas(&llm_client, &narratives).await?;
    info!(count = build_ideas.len(), "build ideas generated");

    // Render HTML report
    let html = output::report::render(&signals, &narratives, &build_ideas)?;
    output::report::write_report(&output_path, &html)?;

    info!(path = %output_path.display(), "report written");
    println!("Report generated: {}", output_path.display());
    println!("  {} signals from {} sources", signals.len(), {
        let sources: std::collections::HashSet<_> = signals.iter().map(|s| s.source).collect();
        sources.len()
    });
    println!("  {} narratives identified", narratives.len());
    println!("  {} build ideas generated", build_ideas.len());

    Ok(())
}

async fn signals_only(config_path: PathBuf) -> Result<()> {
    let cfg = config::Config::load(&config_path)
        .context(format!("loading config from {}", config_path.display()))?;

    let http_client = http::HttpClient::new("st-narrative/0.1.0 (solscout)")?;

    let (github_result, solana_result, social_result) = tokio::join!(
        sources::github::collect(&cfg.github, &http_client),
        sources::solana_rpc::collect(&cfg.solana, &http_client),
        sources::social::collect(&cfg.social, &http_client),
    );

    let mut signals = Vec::new();
    if let Ok(s) = github_result {
        signals.extend(s);
    }
    if let Ok(s) = solana_result {
        signals.extend(s);
    }
    if let Ok(s) = social_result {
        signals.extend(s);
    }

    let json = serde_json::to_string_pretty(&signals)?;
    println!("{json}");

    Ok(())
}
