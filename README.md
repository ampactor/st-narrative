# SolScout Narrative

Autonomous Solana ecosystem intelligence tool that detects emerging narratives from multi-source signals.

## What It Does

SolScout collects signals from three independent sources, aggregates them, and uses LLM synthesis to identify thematic trends backed by real data:

1. **GitHub** — New Solana repositories, star velocity, trending projects, categorized by sector (DeFi, DePIN, AI, NFT, PayFi, Infrastructure, Privacy)
2. **Solana RPC** — Network TPS, epoch state, SOL supply, and per-program transaction rates (paginated for real counts, not capped at 100)
3. **Social/Blogs** — Helius, Solana Foundation, Jito, Marinade blog scraping with Solana relevance filtering

The LLM then identifies 5-8 narratives with confidence scores, trend directions, and quantitative backing. Each narrative requires 2+ signal sources for credibility.

## Output

A self-contained HTML report with:
- Network health metrics (TPS, epoch, supply)
- Per-program activity rates (tx/hr, not just raw counts)
- Identified narratives with confidence and trend indicators
- Actionable build ideas derived from each narrative

**[Live Report](https://ampactor.github.io/st-narrative/)**

## Quick Start

```bash
# Set environment
export GITHUB_TOKEN=...
export OPENROUTER_API_KEY=...       # or ANTHROPIC_API_KEY
export SOLANA_RPC_URL=...           # optional, defaults to public mainnet

# Full pipeline: signals → narratives → build ideas → HTML report
cargo run -- run -c config.toml -o report.html

# Signals only (JSON, no LLM)
cargo run -- signals -c config.toml
```

## Configuration

`config.toml` controls everything: GitHub search parameters, tracked Solana programs, social sources, LLM provider/model. Default uses the free `arcee-ai/trinity-large-preview:free` model on OpenRouter.

## Architecture

```
Sources (parallel)           Analysis              Output
┌─────────────┐
│ GitHub API  │──┐
├─────────────┤  │  ┌────────────┐  ┌─────────────┐  ┌──────────┐
│ Solana RPC  │──┼─→│ Aggregator │─→│ LLM Synth   │─→│ HTML     │
├─────────────┤  │  └────────────┘  └─────────────┘  └──────────┘
│ Blog Scrape │──┘
└─────────────┘
```

## Tech Stack

Rust (edition 2024), reqwest, tokio, clap, askama, scraper, chrono.
