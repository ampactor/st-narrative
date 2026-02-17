use crate::types::{Metric, Signal, SignalSource};
use std::collections::HashMap;

/// Aggregated signal group with computed velocity metrics.
#[derive(Debug, Clone)]
pub struct SignalGroup {
    pub category: String,
    pub signals: Vec<usize>,
    pub source_diversity: usize,
    pub total_signals: usize,
    #[allow(dead_code)]
    pub key_metrics: Vec<Metric>,
}

fn normalize_category(cat: &str) -> String {
    match cat.to_lowercase().as_str() {
        "defi" | "decentralized finance" => "DeFi".into(),
        "nft" | "nfts" | "non-fungible token" | "non-fungible tokens" => "NFT".into(),
        "depin" | "decentralized physical infrastructure" => "DePIN".into(),
        "gaming" | "gamefi" | "game fi" => "Gaming".into(),
        "rwa" | "real world assets" | "real-world assets" => "RWA".into(),
        "dao" | "daos" | "decentralized autonomous organization" => "DAO".into(),
        _ => cat.to_string(),
    }
}

/// Aggregate signals by category, compute cross-source validation.
pub fn aggregate(signals: &[Signal]) -> Vec<SignalGroup> {
    let mut by_category: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, signal) in signals.iter().enumerate() {
        by_category
            .entry(normalize_category(&signal.category))
            .or_default()
            .push(i);
    }

    let mut groups: Vec<SignalGroup> = by_category
        .into_iter()
        .map(|(category, indices)| {
            // Count distinct sources
            let sources: std::collections::HashSet<SignalSource> =
                indices.iter().map(|&i| signals[i].source).collect();

            // Aggregate metrics across signals in this group
            let mut metric_sums: HashMap<String, (f64, String)> = HashMap::new();
            for &i in &indices {
                for m in &signals[i].metrics {
                    let entry = metric_sums
                        .entry(m.name.clone())
                        .or_insert((0.0, m.unit.clone()));
                    entry.0 += m.value;
                }
            }
            let key_metrics: Vec<Metric> = metric_sums
                .into_iter()
                .map(|(name, (value, unit))| Metric { name, value, unit })
                .collect();

            SignalGroup {
                category,
                total_signals: indices.len(),
                source_diversity: sources.len(),
                signals: indices,
                key_metrics,
            }
        })
        .collect();

    // Sort by source diversity (multi-source signals are more credible), then by count
    groups.sort_by(|a, b| {
        b.source_diversity
            .cmp(&a.source_diversity)
            .then(b.total_signals.cmp(&a.total_signals))
    });

    groups
}

/// Prepare a JSON summary of signals for Claude analysis.
pub fn signals_to_json(signals: &[Signal], groups: &[SignalGroup]) -> String {
    let summary: Vec<serde_json::Value> = groups
        .iter()
        .map(|g| {
            let signal_details: Vec<serde_json::Value> = g
                .signals
                .iter()
                .map(|&i| {
                    let s = &signals[i];
                    serde_json::json!({
                        "source": s.source.to_string(),
                        "title": s.title,
                        "description": s.description,
                        "metrics": s.metrics.iter().map(|m| {
                            serde_json::json!({
                                "name": m.name,
                                "value": m.value,
                                "unit": m.unit,
                            })
                        }).collect::<Vec<_>>(),
                        "url": s.url,
                        "timestamp": s.timestamp.to_rfc3339(),
                    })
                })
                .collect();

            serde_json::json!({
                "category": g.category,
                "signal_count": g.total_signals,
                "source_diversity": g.source_diversity,
                "signals": signal_details,
            })
        })
        .collect();

    serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "[]".into())
}
