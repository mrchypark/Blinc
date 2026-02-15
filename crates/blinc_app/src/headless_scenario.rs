//! Scenario definition for app-level headless diagnostics.

use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

/// Sequence of headless diagnostic steps.
#[derive(Debug, Clone, Deserialize)]
pub struct HeadlessScenario {
    pub steps: Vec<ScenarioStep>,
}

impl HeadlessScenario {
    /// Load a scenario from JSON text.
    pub fn from_json(input: &str) -> Result<Self> {
        Ok(serde_json::from_str(input)?)
    }

    /// Load a scenario from file.
    pub fn from_path(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        Self::from_json(&raw)
    }
}

/// Minimal scenario step set for the diagnostics MVP.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScenarioStep {
    Wait { ms: u64 },
    Tick { frames: u32 },
    AssertExists { id: String },
    AssertTextContains { id: String, value: String },
}
