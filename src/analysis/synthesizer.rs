use crate::error::Result;
use crate::llm::LlmClient;
use crate::types::{Metric, Narrative, TrendDirection};
use serde::Deserialize;
use tracing::info;

const SYSTEM_PROMPT: &str = r#"You are a senior Solana ecosystem analyst identifying emerging narratives from cross-source signal data.

A "narrative" is a thematic trend backed by multiple data points across different sources (GitHub developer activity, onchain metrics, DeFi TVL, social/blog signals). A narrative must appear in 2+ signal sources to be credible.

For each narrative you identify, provide:
1. A clear, specific title — name the specific protocols, tools, or primitives involved. "Concentrated Liquidity Migration on Raydium and Orca" not "DeFi growth."
2. A 2-3 sentence summary covering: what is happening, why it matters for the Solana ecosystem, and what structural shift it represents.
3. Confidence score (0.0-1.0) based on signal strength and source diversity.
4. Which signal indices support this narrative (from the input data).
5. Trend direction: "Accelerating" (growing faster), "Stable" (steady), "Decelerating" (slowing), "Emerging" (too early to tell, but signals present).
6. Key quantitative metrics that back the narrative.

Analysis depth requirements:
- **Historical context:** Is this a new trend or continuation of an existing one? What would be unusual or surprising about these numbers?
- **Structural implications:** What does this trend enable or threaten in the ecosystem? Which protocols or categories benefit or lose?
- **Cross-signal validation:** Do GitHub activity, onchain metrics, TVL data, and social signals agree? Explicitly flag divergences (e.g., rising developer activity but flat TVL suggests pre-launch building).
- **Second-order effects:** What follows from this trend? If liquid staking is growing, what does that unlock for DeFi composability?
- **Specificity:** Name specific protocols, repositories, and programs. Reference actual addresses, repo names, and TVL figures from the data.

Respond in JSON:
{
  "narratives": [
    {
      "title": "...",
      "summary": "...",
      "confidence": 0.85,
      "supporting_signals": [0, 3, 7],
      "trend": "Accelerating",
      "key_metrics": [{"name": "...", "value": 123.4, "unit": "..."}]
    }
  ]
}

Rules:
- Only report narratives you're confident about. Quality over quantity.
- Every claim must be backed by specific signals from the input data.
- Quantify everything. "Growing" is weak; "42% increase in new repos" is strong.
- 5-8 narratives is ideal. Fewer if the data doesn't support more.
- Don't invent data. Only use what's in the signals.
- When signals contradict each other, say so — contradiction is itself a signal."#;

#[derive(Deserialize)]
struct SynthesisResponse {
    narratives: Vec<RawNarrative>,
}

#[derive(Deserialize)]
struct RawNarrative {
    title: String,
    summary: String,
    confidence: f64,
    supporting_signals: Vec<usize>,
    trend: String,
    #[serde(default)]
    key_metrics: Vec<RawMetric>,
}

#[derive(Deserialize)]
struct RawMetric {
    name: String,
    value: f64,
    #[serde(default)]
    unit: String,
}

pub async fn identify_narratives(llm: &LlmClient, signals_json: &str) -> Result<Vec<Narrative>> {
    info!("sending signals to LLM for narrative identification");

    let user_message = format!(
        "Analyze these aggregated signals from the Solana ecosystem and identify emerging narratives:\n\n{signals_json}"
    );

    let response: SynthesisResponse = llm.complete_json(SYSTEM_PROMPT, &user_message).await?;

    let count = response.narratives.len();
    let narratives = response
        .narratives
        .into_iter()
        .map(|n| Narrative {
            title: n.title,
            summary: n.summary,
            confidence: n.confidence.clamp(0.0, 1.0),
            supporting_signals: n.supporting_signals,
            trend: parse_trend(&n.trend),
            key_metrics: n
                .key_metrics
                .into_iter()
                .map(|m| Metric {
                    name: m.name,
                    value: m.value,
                    unit: m.unit,
                })
                .collect(),
        })
        .collect();

    info!(count, "identified narratives");
    Ok(narratives)
}

fn parse_trend(s: &str) -> TrendDirection {
    match s.to_lowercase().as_str() {
        "accelerating" => TrendDirection::Accelerating,
        "stable" | "steady" => TrendDirection::Stable,
        "decelerating" | "declining" => TrendDirection::Decelerating,
        "emerging" | "nascent" | "early" => TrendDirection::Emerging,
        _ => TrendDirection::Emerging,
    }
}
