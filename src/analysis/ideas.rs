use crate::claude::ClaudeClient;
use crate::error::Result;
use crate::types::{BuildIdea, Narrative};
use serde::Deserialize;
use tracing::info;

const SYSTEM_PROMPT: &str = r#"You are a product strategist for the Solana ecosystem. Given identified narratives with supporting data, generate concrete build ideas that an AI agent or small team could implement in one week.

For each build idea, provide:
1. A specific product name/title
2. Clear description of what it does
3. Target user (who uses this and why)
4. MVP scope (what you build in a week — be realistic)
5. Competitive landscape (what exists, what's missing)
6. Timing rationale (why now, not 6 months ago or 6 months from now)
7. Which narrative index this idea supports (from the input)

Generate 3-5 ideas per narrative. Focus on ideas that are:
- Immediately useful (not "build a protocol" — think tools, dashboards, bots)
- Differentiated (not another DEX aggregator)
- Feasible for an AI agent to prototype

Respond in JSON:
{
  "ideas": [
    {
      "title": "...",
      "description": "...",
      "target_user": "...",
      "mvp_scope": "...",
      "competitive_landscape": "...",
      "timing_rationale": "...",
      "narrative_index": 0
    }
  ]
}"#;

#[derive(Deserialize)]
struct IdeasResponse {
    ideas: Vec<RawIdea>,
}

#[derive(Deserialize)]
struct RawIdea {
    title: String,
    description: String,
    target_user: String,
    mvp_scope: String,
    competitive_landscape: String,
    timing_rationale: String,
    narrative_index: usize,
}

pub async fn generate_ideas(
    claude: &ClaudeClient,
    narratives: &[Narrative],
) -> Result<Vec<BuildIdea>> {
    info!(narrative_count = narratives.len(), "generating build ideas");

    let narratives_json = serde_json::to_string_pretty(narratives).unwrap_or_else(|_| "[]".into());

    let user_message =
        format!("Generate build ideas for these Solana ecosystem narratives:\n\n{narratives_json}");

    let response: IdeasResponse = claude.complete_json(SYSTEM_PROMPT, &user_message).await?;

    let count = response.ideas.len();
    let ideas = response
        .ideas
        .into_iter()
        .map(|i| BuildIdea {
            title: i.title,
            description: i.description,
            target_user: i.target_user,
            mvp_scope: i.mvp_scope,
            competitive_landscape: i.competitive_landscape,
            timing_rationale: i.timing_rationale,
            narrative_index: i.narrative_index,
        })
        .collect();

    info!(count, "generated build ideas");
    Ok(ideas)
}
