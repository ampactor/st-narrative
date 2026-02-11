use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub source: SignalSource,
    pub category: String,
    pub title: String,
    pub description: String,
    pub metrics: Vec<Metric>,
    pub url: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalSource {
    GitHub,
    SolanaOnchain,
    Social,
}

impl std::fmt::Display for SignalSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "GitHub"),
            Self::SolanaOnchain => write!(f, "Solana Onchain"),
            Self::Social => write!(f, "Social"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
}

impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.unit.is_empty() {
            write!(f, "{}: {:.1}", self.name, self.value)
        } else {
            write!(f, "{}: {:.1} {}", self.name, self.value, self.unit)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narrative {
    pub title: String,
    pub summary: String,
    pub confidence: f64,
    pub supporting_signals: Vec<usize>,
    pub trend: TrendDirection,
    pub key_metrics: Vec<Metric>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Accelerating,
    Stable,
    Decelerating,
    Emerging,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accelerating => write!(f, "Accelerating"),
            Self::Stable => write!(f, "Stable"),
            Self::Decelerating => write!(f, "Decelerating"),
            Self::Emerging => write!(f, "Emerging"),
        }
    }
}

impl TrendDirection {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Accelerating => "text-green-400",
            Self::Stable => "text-blue-400",
            Self::Decelerating => "text-red-400",
            Self::Emerging => "text-yellow-400",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildIdea {
    pub title: String,
    pub description: String,
    pub target_user: String,
    pub mvp_scope: String,
    pub competitive_landscape: String,
    pub timing_rationale: String,
    pub narrative_index: usize,
}
