# SolScout — Solana Narrative Detection Tool

SolScout is a refreshable intelligence tool that detects emerging narratives in the Solana ecosystem by cross-validating signals from four independent data sources, then synthesizing them into actionable build ideas via LLM analysis.

**[Live Report](https://ampactor.github.io/st-narrative/)** — auto-refreshed on the 1st and 15th of each month.

## Signal Sources

SolScout collects signals in parallel from four sources. Each provides a different lens on ecosystem activity:

| Source | What It Provides | Why It Matters |
|--------|-----------------|----------------|
| **GitHub API** | New Solana repos, star velocity, trending projects, sector categorization (DeFi, DePIN, AI, NFT, PayFi, Infrastructure, Privacy) | Developer attention is a leading indicator — what builders invest time in predicts ecosystem direction 2-6 months out |
| **Solana RPC** | Network TPS, epoch state, SOL supply, per-program transaction rates (paginated for real counts) | On-chain activity is ground truth — it shows what users actually do vs. what narratives claim |
| **Blog Scraping** | Articles from Helius, Solana Foundation, Jito, Marinade with Solana relevance filtering | Ecosystem players telegraph strategy through blog posts — these are soft signals that precede on-chain shifts |
| **DeFiLlama** | Solana chain TVL, top protocol TVL rankings, category breakdowns | TVL tracks capital allocation — money follows conviction, and TVL shifts reveal which narratives have financial backing |

## Methodology

1. **Parallel collection** — All four sources run concurrently via Tokio tasks. Each produces typed `Signal` structs with source attribution, category, metrics, and optional URLs.

2. **Aggregation** — Signals are grouped by category (DeFi, Infrastructure, AI, etc.) and scored for source diversity. A narrative backed by GitHub activity + on-chain data + blog coverage is stronger than one from a single source.

3. **Cross-validation** — The aggregator computes derived metrics (tx/hr rates, ratios between programs, star velocity) that reveal patterns invisible in raw counts.

4. **LLM synthesis** — All aggregated signals are passed to Claude with a structured prompt that requires:
   - Each narrative to cite specific signal indices and quantitative metrics
   - Confidence scores reflecting source diversity and metric strength
   - Trend direction (accelerating/stable/emerging) with justification
   - Cross-source corroboration — narratives need 2+ source types

5. **Idea generation** — A second LLM pass takes identified narratives and generates build ideas, each with target user, MVP scope, competitive landscape, and timing rationale grounded in the signal data.

## Explainability: How a Narrative Is Built

Every narrative in the report is traceable back to specific signals. Here is a concrete example from the latest run:

> **Jupiter Dominance as Solana's DeFi Routing Layer** (88% confidence, Accelerating)
>
> Supporting metrics:
> - `jupiter_tx_per_hour: 211,765 tx/hr` (Solana RPC — program activity)
> - `raydium_tx_per_hour: 25,000 tx/hr` (Solana RPC — program activity)
> - `jupiter_to_raydium_ratio: 8.5x` (derived metric)
> - `defi_new_repos: 9 repos, 269 stars, 229 forks` (GitHub API — sector count)
>
> **Why 88% confidence:** Three source types corroborate — on-chain tx rates show Jupiter dominance quantitatively, GitHub shows DeFi tooling building around Jupiter's flow, and the 8.5x ratio is a derived cross-validation metric. Blog posts from Helius discuss Jupiter integration patterns.

The report includes a full raw signals table so readers can verify any narrative's claims against the underlying data.

## Example Build Ideas

From the latest report — each idea is grounded in detected narratives and their quantitative backing:

1. **JupiterScope** — Real-time route efficiency analyzer that scores whether Jupiter's routing is optimal vs. direct AMM access. Backed by the 211K tx/hr Jupiter throughput signal.

2. **SandwichRadar** — MEV attack detection and alerting. Emerges from the anti-JIT/MEV protection tooling narrative with 3 new security-focused repos.

3. **AgentLedger** — On-chain AI agent activity dashboard tracking SIGNIA and x402 protocol adoption. Driven by the AI agent infrastructure narrative.

4. **EpochPulse** — Solana epoch analytics and staking reward predictor. Built on the Firedancer client maturation narrative and network health signals.

5. **LaunchGuard** — Memecoin launch risk scanner using on-chain patterns. Sourced from the token launch tooling proliferation narrative with 4 new repos.

The full report contains 25 build ideas across 8 detected narratives, each with target users, MVP scope, competitive analysis, and timing rationale.

## Running Locally

```bash
# Required environment variables
export GITHUB_TOKEN=...
export ANTHROPIC_API_KEY=...        # or OPENROUTER_API_KEY for free models
export SOLANA_RPC_URL=...           # optional, defaults to public mainnet

# Full pipeline: collect signals -> detect narratives -> generate ideas -> HTML report
cargo run -- run -c config.toml -o report.html

# Signals only (JSON output, no LLM cost)
cargo run -- signals -c config.toml

# Use Anthropic provider explicitly
cargo run -- run -c config.toml --provider anthropic -o report.html
```

`config.toml` controls: GitHub search parameters, tracked Solana programs, blog sources, DeFiLlama settings, LLM provider/model selection.

## Automated Refresh

The GitHub Actions workflow rebuilds the report on a fortnightly schedule:

- **Cron:** `0 0 1,15 * *` — runs at midnight UTC on the 1st and 15th of each month
- **Push:** any push to `main` triggers a fresh build
- **Manual:** `workflow_dispatch` allows on-demand regeneration

Each run: checkout -> build Rust release -> execute full pipeline -> deploy to GitHub Pages. Secrets required: `ANTHROPIC_API_KEY`, `SOLANA_RPC_URL`.

## Architecture

```
Sources (parallel)           Analysis              Output
+--------------+
| GitHub API   |--+
+--------------+  |  +------------+  +-----------+  +--------+
| Solana RPC   |--+->| Aggregator |->| LLM Synth |->|  HTML  |
+--------------+  |  +------------+  +-----------+  +--------+
| Blog Scrape  |--+
+--------------+  |
| DeFiLlama    |--+
+--------------+
```

Rust (edition 2024), reqwest, tokio, clap, askama, scraper, chrono.
