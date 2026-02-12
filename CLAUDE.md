# st-narrative

Solana narrative detection and idea generation tool for SuperTeam Earn bounty.

## Build & Test

```bash
cargo build
cargo test
cargo clippy -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo test --doc
```

## Run

```bash
# Full pipeline: collect signals → Claude analysis → HTML report
cargo run -- run -c config.toml -o report.html

# Signals only (no Claude, outputs JSON)
cargo run -- signals -c config.toml
```

## Environment Variables

```bash
GITHUB_TOKEN=          # GitHub API access
OPENROUTER_API_KEY=    # Default LLM provider for narrative synthesis
ANTHROPIC_API_KEY=     # When using provider = "anthropic" in config.toml
HELIUS_API_KEY=        # (optional) Helius RPC
SOLANA_RPC_URL=        # Solana RPC endpoint (default: public mainnet)
```

Shared .env at `~/Documents/.env` — loaded automatically.

## Architecture

```
CLI (clap) → Sources (parallel) → Aggregator → Claude Synthesis → HTML Report (Askama)
```

### Sources
- `sources/github.rs` — GitHub Search API: new Solana repos, star velocity, trending
- `sources/solana_rpc.rs` — Helius/Solana RPC: TPS, epoch info, program activity
- `sources/social.rs` — Blog scraping: article extraction, Solana relevance filtering

### Analysis
- `analysis/aggregator.rs` — Group signals by category, compute source diversity
- `analysis/synthesizer.rs` — LLM: signals → narrative identification
- `analysis/ideas.rs` — LLM: narratives → build ideas

### Output
- `output/report.rs` — Askama template rendering → static HTML
- `templates/report.html` — Tailwind dark-mode report template

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry, pipeline orchestration |
| `src/config.rs` | TOML + env var config loading |
| `src/http.rs` | HTTP client with retry/backoff |
| `src/llm.rs` | Provider-swappable LLM client (Anthropic/OpenRouter/OpenAI) |
| `src/types.rs` | Signal, Narrative, BuildIdea types |
| `config.toml` | Default configuration |
| `templates/report.html` | HTML report template |

## Sprint Context

Part of SuperTeam bounty sprint (Feb 11-15, 2026).
Durable state: `~/.claude/projects/-home-suds-Documents/memory/superteam-sprint.md`
Master plan: `~/.claude/plans/eager-sleeping-minsky.md`

## Doc-to-Code Mapping

| Source File(s) | Documentation Target(s) | What to Update |
|---|---|---|
| `src/main.rs`, `src/config.rs` | README.md, CLAUDE.md | CLI usage, config options |
| `src/sources/*.rs` | CLAUDE.md (Architecture) | Data source details |
| `src/analysis/*.rs` | CLAUDE.md (Architecture) | Analysis pipeline |
| `src/types.rs` | CLAUDE.md (Key Files) | Type definitions |
| `templates/report.html` | CLAUDE.md (Output) | Template structure |
