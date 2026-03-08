//! Scenario definition for app-level headless diagnostics.

use anyhow::{bail, Result};
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
        let scenario: Self = serde_json::from_str(input)?;
        scenario.validate()?;
        Ok(scenario)
    }

    /// Load a scenario from file.
    pub fn from_path(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        Self::from_json(&raw)
    }

    pub fn validate(&self) -> Result<()> {
        for step in &self.steps {
            match step {
                ScenarioStep::Click { target, x, y } => {
                    let has_point = x.is_some() || y.is_some();
                    if has_point {
                        if x.is_none() || y.is_none() {
                            bail!("coordinate click steps require both x and y");
                        }
                        if !target.is_empty() {
                            bail!("coordinate click steps cannot mix locator fields with x/y");
                        }
                    } else {
                        target
                            .validate_required()
                            .map_err(|err| anyhow::anyhow!("{err}"))?;
                    }
                }
                ScenarioStep::Fill { target, .. }
                | ScenarioStep::AssertExists { target }
                | ScenarioStep::AssertTextContains { target, .. } => target
                    .validate_required()
                    .map_err(|err| anyhow::anyhow!("{err}"))?,
                ScenarioStep::Scroll { target, .. } => target.validate_optional()?,
                ScenarioStep::Wait { .. }
                | ScenarioStep::Tick { .. }
                | ScenarioStep::Press { .. }
                | ScenarioStep::Snapshot { .. }
                | ScenarioStep::ExportTrace { .. } => {}
            }
        }
        Ok(())
    }

    pub fn resolve_embedded_paths(&mut self, base: &Path) {
        for step in &mut self.steps {
            step.resolve_artifact_paths(base);
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ScenarioTarget {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(flatten)]
    pub semantic: ScenarioSemanticLocator,
}

impl ScenarioTarget {
    pub fn is_empty(&self) -> bool {
        self.id.is_none() && !self.has_semantic_fields()
    }

    pub fn has_semantic_fields(&self) -> bool {
        self.semantic.has_any()
    }

    pub fn validate_required(&self) -> Result<()> {
        let has_id = self.id.is_some();
        let has_semantic = self.has_semantic_fields();
        match (has_id, has_semantic) {
            (true, false) | (false, true) => Ok(()),
            (false, false) => bail!("scenario step requires either id or semantic locator fields"),
            (true, true) => {
                bail!("scenario step cannot mix id with semantic locator fields")
            }
        }
    }

    pub fn validate_optional(&self) -> Result<()> {
        let has_id = self.id.is_some();
        let has_semantic = self.has_semantic_fields();
        if has_id && has_semantic {
            bail!("scenario step cannot mix id with semantic locator fields");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ScenarioSemanticLocator {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub within: Option<String>,
    #[serde(default)]
    pub nth: Option<usize>,
}

impl ScenarioSemanticLocator {
    pub fn has_any(&self) -> bool {
        self.role.is_some()
            || self.text.is_some()
            || self.label.is_some()
            || self.placeholder.is_some()
            || self.tag.is_some()
            || self.within.is_some()
            || self.nth.is_some()
    }
}

/// Minimal scenario step set for the diagnostics MVP.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScenarioStep {
    Wait {
        ms: u64,
    },
    Tick {
        frames: u32,
    },
    Click {
        #[serde(flatten)]
        target: ScenarioTarget,
        #[serde(default)]
        x: Option<f32>,
        #[serde(default)]
        y: Option<f32>,
    },
    Fill {
        #[serde(flatten)]
        target: ScenarioTarget,
        value: String,
    },
    Press {
        key: String,
    },
    Scroll {
        #[serde(flatten)]
        target: ScenarioTarget,
        dx: f32,
        dy: f32,
    },
    Snapshot {
        path: Option<String>,
    },
    ExportTrace {
        path: Option<String>,
    },
    AssertExists {
        #[serde(flatten)]
        target: ScenarioTarget,
    },
    AssertTextContains {
        #[serde(flatten)]
        target: ScenarioTarget,
        value: String,
    },
}

impl ScenarioStep {
    pub(crate) fn resolve_artifact_paths(&mut self, base: &Path) {
        let path = match self {
            ScenarioStep::Snapshot { path } | ScenarioStep::ExportTrace { path } => path,
            _ => return,
        };

        if let Some(value) = path {
            let candidate = Path::new(value);
            if candidate.is_relative() {
                *value = base.join(candidate).display().to_string();
            }
        }
    }
}
