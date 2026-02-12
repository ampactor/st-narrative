use crate::error::{Error, Result};
use crate::types::{BuildIdea, Narrative, Signal};
use askama::Template;
use chrono::Utc;
use std::path::Path;

#[derive(Template)]
#[template(path = "report.html")]
pub struct ReportTemplate {
    pub generated_at: String,
    pub total_signals: usize,
    pub source_count: usize,
    pub narratives: Vec<NarrativeView>,
    pub build_ideas: Vec<BuildIdeaView>,
    pub signals: Vec<SignalView>,
}

pub struct NarrativeView {
    pub title: String,
    pub summary: String,
    pub confidence_pct: u32,
    pub trend: String,
    pub trend_class: String,
    pub signal_count: usize,
    pub metrics: Vec<String>,
}

pub struct BuildIdeaView {
    pub title: String,
    pub description: String,
    pub target_user: String,
    pub mvp_scope: String,
    pub competitive_landscape: String,
    pub timing_rationale: String,
    pub narrative_title: String,
}

#[allow(dead_code)] // fields used by Askama template
pub struct SignalView {
    pub source: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub metrics: Vec<String>,
    pub url: String,
}

pub fn render(
    signals: &[Signal],
    narratives: &[Narrative],
    build_ideas: &[BuildIdea],
) -> Result<String> {
    let sources: std::collections::HashSet<_> = signals.iter().map(|s| s.source).collect();

    let narrative_views: Vec<NarrativeView> = narratives
        .iter()
        .map(|n| NarrativeView {
            title: n.title.clone(),
            summary: n.summary.clone(),
            confidence_pct: (n.confidence * 100.0) as u32,
            trend: n.trend.to_string(),
            trend_class: n.trend.css_class().to_string(),
            signal_count: n.supporting_signals.len(),
            metrics: n.key_metrics.iter().map(|m| m.to_string()).collect(),
        })
        .collect();

    let idea_views: Vec<BuildIdeaView> = build_ideas
        .iter()
        .map(|i| BuildIdeaView {
            title: i.title.clone(),
            description: i.description.clone(),
            target_user: i.target_user.clone(),
            mvp_scope: i.mvp_scope.clone(),
            competitive_landscape: i.competitive_landscape.clone(),
            timing_rationale: i.timing_rationale.clone(),
            narrative_title: narratives
                .get(i.narrative_index)
                .map(|n| n.title.clone())
                .unwrap_or_else(|| "Unknown".into()),
        })
        .collect();

    let signal_views: Vec<SignalView> = signals
        .iter()
        .map(|s| SignalView {
            source: s.source.to_string(),
            category: s.category.clone(),
            title: s.title.clone(),
            description: s.description.clone(),
            metrics: s.metrics.iter().map(|m| m.to_string()).collect(),
            url: s.url.clone().unwrap_or_default(),
        })
        .collect();

    let template = ReportTemplate {
        generated_at: Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        total_signals: signals.len(),
        source_count: sources.len(),
        narratives: narrative_views,
        build_ideas: idea_views,
        signals: signal_views,
    };

    template
        .render()
        .map_err(|e| Error::Template(e.to_string()))
}

pub fn write_report(path: &Path, html: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, html)?;
    Ok(())
}
