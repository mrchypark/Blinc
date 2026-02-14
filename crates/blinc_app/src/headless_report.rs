//! Report output model for headless diagnostics runs.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Component, Path};

/// Report status for a headless diagnostics run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Passed,
    Failed,
}

/// Machine-readable result of a headless diagnostics run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadlessReport {
    pub status: ReportStatus,
    pub failed_step_index: Option<usize>,
    pub assertion: Option<String>,
    pub message: Option<String>,
    pub elapsed_frames: u64,
    pub elapsed_ms: u64,
}

impl HeadlessReport {
    pub fn passed(elapsed_frames: u64, elapsed_ms: u64) -> Self {
        Self {
            status: ReportStatus::Passed,
            failed_step_index: None,
            assertion: None,
            message: None,
            elapsed_frames,
            elapsed_ms,
        }
    }

    pub fn failed(
        assertion: &str,
        failed_step_index: usize,
        message: String,
        elapsed_frames: u64,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            status: ReportStatus::Failed,
            failed_step_index: Some(failed_step_index),
            assertion: Some(assertion.to_string()),
            message: Some(message),
            elapsed_frames,
            elapsed_ms,
        }
    }

    pub fn write_to_path(&self, path: &Path) -> Result<()> {
        if path.is_absolute() || path.has_root() {
            bail!("report path must be relative and must not start with a separator");
        }
        if path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            bail!("report path cannot contain '..' or drive prefixes");
        }
        let payload = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    pub fn write_to_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
        let payload = serde_json::to_string_pretty(self)?;
        writer.write_all(payload.as_bytes())?;
        writer.write_all(b"\n")?;
        Ok(())
    }
}
