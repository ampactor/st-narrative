use crate::config::SolanaConfig;
use crate::error::{Error, Result};
use crate::http::HttpClient;
use crate::types::{Metric, Signal, SignalSource};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: u64,
    method: &'a str,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct PerformanceSample {
    #[serde(rename = "numTransactions")]
    num_transactions: u64,
    #[serde(rename = "numNonVoteTransactions")]
    num_non_vote_transactions: Option<u64>,
    #[serde(rename = "numSlots")]
    num_slots: u64,
    #[serde(rename = "samplePeriodSecs")]
    sample_period_secs: u64,
}

#[derive(Deserialize)]
struct EpochInfo {
    epoch: u64,
    #[serde(rename = "slotIndex")]
    slot_index: u64,
    #[serde(rename = "slotsInEpoch")]
    slots_in_epoch: u64,
    #[serde(rename = "absoluteSlot")]
    absolute_slot: u64,
    #[serde(rename = "transactionCount")]
    transaction_count: Option<u64>,
}

#[derive(Deserialize)]
struct Supply {
    value: SupplyValue,
}

#[derive(Deserialize)]
struct SupplyValue {
    total: u64,
    circulating: u64,
    #[serde(rename = "nonCirculating")]
    non_circulating: u64,
}

pub async fn collect(config: &SolanaConfig, http: &HttpClient) -> Result<Vec<Signal>> {
    let mut signals = Vec::new();

    // Get recent performance samples (TPS data)
    let perf_samples = rpc_call::<Vec<PerformanceSample>>(
        &config.rpc_url,
        http,
        "getRecentPerformanceSamples",
        serde_json::json!([10]),
    )
    .await?;

    if !perf_samples.is_empty() {
        let avg_tps: f64 = perf_samples
            .iter()
            .map(|s| s.num_transactions as f64 / s.sample_period_secs as f64)
            .sum::<f64>()
            / perf_samples.len() as f64;

        let avg_non_vote_tps: f64 = perf_samples
            .iter()
            .filter_map(|s| {
                s.num_non_vote_transactions
                    .map(|nv| nv as f64 / s.sample_period_secs as f64)
            })
            .sum::<f64>()
            / perf_samples.len() as f64;

        signals.push(Signal {
            source: SignalSource::SolanaOnchain,
            category: "Network Performance".into(),
            title: format!("Solana TPS: {avg_tps:.0} total, {avg_non_vote_tps:.0} non-vote"),
            description: format!(
                "Average over {} recent samples. Non-vote TPS indicates real user activity vs consensus overhead.",
                perf_samples.len()
            ),
            metrics: vec![
                Metric { name: "avg_tps".into(), value: avg_tps, unit: "tx/s".into() },
                Metric { name: "avg_non_vote_tps".into(), value: avg_non_vote_tps, unit: "tx/s".into() },
            ],
            url: Some("https://explorer.solana.com/".into()),
            timestamp: Utc::now(),
        });
    }

    // Get epoch info
    let epoch: EpochInfo =
        rpc_call(&config.rpc_url, http, "getEpochInfo", serde_json::json!([])).await?;

    let epoch_progress = epoch.slot_index as f64 / epoch.slots_in_epoch as f64 * 100.0;

    signals.push(Signal {
        source: SignalSource::SolanaOnchain,
        category: "Network State".into(),
        title: format!("Epoch {} — {epoch_progress:.1}% complete", epoch.epoch),
        description: format!(
            "Slot {}/{}, absolute slot {}. {}",
            epoch.slot_index,
            epoch.slots_in_epoch,
            epoch.absolute_slot,
            epoch
                .transaction_count
                .map(|tc| format!("Total transactions: {tc}"))
                .unwrap_or_default()
        ),
        metrics: vec![
            Metric {
                name: "epoch".into(),
                value: epoch.epoch as f64,
                unit: String::new(),
            },
            Metric {
                name: "epoch_progress".into(),
                value: epoch_progress,
                unit: "%".into(),
            },
            Metric {
                name: "absolute_slot".into(),
                value: epoch.absolute_slot as f64,
                unit: "slot".into(),
            },
        ],
        url: Some("https://explorer.solana.com/".into()),
        timestamp: Utc::now(),
    });

    // Get SOL supply
    let supply: Supply =
        rpc_call(&config.rpc_url, http, "getSupply", serde_json::json!([])).await?;

    let circulating_pct = supply.value.circulating as f64 / supply.value.total as f64 * 100.0;

    signals.push(Signal {
        source: SignalSource::SolanaOnchain,
        category: "Token Economics".into(),
        title: format!(
            "SOL Supply: {:.1}M circulating ({circulating_pct:.1}%)",
            supply.value.circulating as f64 / 1_000_000_000.0 / 1_000_000.0,
        ),
        description: format!(
            "Total: {:.1}M SOL, Circulating: {:.1}M SOL, Non-circulating: {:.1}M SOL",
            supply.value.total as f64 / 1e15,
            supply.value.circulating as f64 / 1e15,
            supply.value.non_circulating as f64 / 1e15,
        ),
        metrics: vec![
            Metric {
                name: "circulating_sol".into(),
                value: supply.value.circulating as f64 / 1e9,
                unit: "SOL".into(),
            },
            Metric {
                name: "circulating_pct".into(),
                value: circulating_pct,
                unit: "%".into(),
            },
        ],
        url: None,
        timestamp: Utc::now(),
    });

    // Get signature counts for tracked programs
    for program in &config.tracked_programs {
        match get_program_activity(&config.rpc_url, http, &program.address).await {
            Ok(sig_count) => {
                signals.push(Signal {
                    source: SignalSource::SolanaOnchain,
                    category: program.category.clone(),
                    title: format!("{}: {} recent transactions", program.name, sig_count),
                    description: format!(
                        "Program {} ({}) had {sig_count} transactions in recent history.",
                        program.name, program.address
                    ),
                    metrics: vec![Metric {
                        name: "recent_tx_count".into(),
                        value: sig_count as f64,
                        unit: "txs".into(),
                    }],
                    url: Some(format!(
                        "https://explorer.solana.com/address/{}",
                        program.address
                    )),
                    timestamp: Utc::now(),
                });
            }
            Err(e) => {
                tracing::warn!(program = %program.name, error = %e, "failed to get program activity");
            }
        }
    }

    info!(
        signal_count = signals.len(),
        "collected Solana onchain signals"
    );
    Ok(signals)
}

async fn get_program_activity(rpc_url: &str, http: &HttpClient, address: &str) -> Result<usize> {
    #[derive(Deserialize)]
    struct SigInfo {
        #[allow(dead_code)]
        signature: String,
    }

    let sigs: Vec<SigInfo> = rpc_call(
        rpc_url,
        http,
        "getSignaturesForAddress",
        serde_json::json!([address, {"limit": 100}]),
    )
    .await?;

    Ok(sigs.len())
}

async fn rpc_call<T: serde::de::DeserializeOwned>(
    rpc_url: &str,
    http: &HttpClient,
    method: &str,
    params: serde_json::Value,
) -> Result<T> {
    let request = RpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method,
        params,
    };

    let body =
        serde_json::to_string(&request).map_err(|e| Error::parse(format!("serialize: {e}")))?;

    let resp_text = http.post_json_raw(rpc_url, &body, &[]).await?;

    let resp: RpcResponse<T> =
        serde_json::from_str(&resp_text).map_err(|e| Error::parse(format!("parse RPC: {e}")))?;

    if let Some(err) = resp.error {
        return Err(Error::api("solana-rpc", err.message));
    }

    resp.result
        .ok_or_else(|| Error::parse("RPC response missing result"))
}
