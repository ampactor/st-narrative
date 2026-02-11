use crate::config::SocialConfig;
use crate::error::Result;
use crate::http::HttpClient;
use crate::types::{Metric, Signal, SignalSource};
use chrono::Utc;
use scraper::{Html, Selector};
use tracing::{info, warn};

pub async fn collect(config: &SocialConfig, http: &HttpClient) -> Result<Vec<Signal>> {
    let mut signals = Vec::new();

    for source in &config.sources {
        match scrape_source(http, &source.name, &source.url).await {
            Ok(mut source_signals) => signals.append(&mut source_signals),
            Err(e) => {
                warn!(source = %source.name, url = %source.url, error = %e, "failed to scrape, skipping");
            }
        }
    }

    info!(signal_count = signals.len(), "collected social signals");
    Ok(signals)
}

async fn scrape_source(http: &HttpClient, name: &str, url: &str) -> Result<Vec<Signal>> {
    let html_text = http.get_text(url).await?;
    let document = Html::parse_document(&html_text);

    // Extract article titles and links — generic selectors that work for most blogs
    let selectors = [
        "article h2 a",
        "article h3 a",
        ".post-title a",
        "h2.entry-title a",
        "a[class*='title']",
        "h2 a",
        "h3 a",
    ];

    let mut articles = Vec::new();

    for sel_str in &selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for element in document.select(&selector) {
                let title = element.text().collect::<String>().trim().to_string();
                let href = element.value().attr("href").unwrap_or("").to_string();

                if !title.is_empty() && title.len() > 5 {
                    articles.push((title, href));
                }
            }
            if !articles.is_empty() {
                break; // found articles with this selector, stop trying
            }
        }
    }

    // Deduplicate by title
    articles.sort_by(|a, b| a.0.cmp(&b.0));
    articles.dedup_by(|a, b| a.0 == b.0);

    let solana_articles: Vec<&(String, String)> = articles
        .iter()
        .filter(|(title, _)| {
            let lower = title.to_lowercase();
            lower.contains("solana")
                || lower.contains("sol")
                || lower.contains("defi")
                || lower.contains("depin")
                || lower.contains("token")
                || lower.contains("validator")
                || lower.contains("staking")
                || lower.contains("nft")
                || lower.contains("web3")
                || lower.contains("blockchain")
                || lower.contains("crypto")
        })
        .collect();

    if solana_articles.is_empty() && articles.is_empty() {
        return Ok(Vec::new());
    }

    let relevant = if solana_articles.is_empty() {
        &articles
    } else {
        // Use solana_articles but we need owned references
        // Just use all articles and note the Solana-relevant count
        &articles
    };

    let titles: Vec<String> = relevant.iter().take(10).map(|(t, _)| t.clone()).collect();

    Ok(vec![Signal {
        source: SignalSource::Social,
        category: format!("Blog: {name}"),
        title: format!(
            "{name}: {} recent articles ({} Solana-related)",
            articles.len(),
            solana_articles.len()
        ),
        description: format!("Recent topics: {}", titles.join("; ")),
        metrics: vec![
            Metric {
                name: "total_articles".into(),
                value: articles.len() as f64,
                unit: "articles".into(),
            },
            Metric {
                name: "solana_relevant".into(),
                value: solana_articles.len() as f64,
                unit: "articles".into(),
            },
        ],
        url: Some(url.to_string()),
        timestamp: Utc::now(),
    }])
}
