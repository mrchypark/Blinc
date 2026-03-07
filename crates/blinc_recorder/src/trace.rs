//! Trace records layered on top of recorder exports.

use crate::Timestamp;
use serde::{Deserialize, Serialize};

/// A single trace entry in an exported recording.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEntry {
    pub sequence: u64,
    pub timestamp: Timestamp,
    pub kind: TraceEntryKind,
}

/// Trace entry variants captured alongside events and snapshots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceEntryKind {
    Command(TraceCommandRecord),
    LocatorResolution(TraceLocatorResolution),
    Assertion(TraceAssertionRecord),
    Artifact(TraceArtifactRecord),
}

/// Canonical command issued by automation or scenario execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceCommandRecord {
    pub name: String,
    pub target: Option<String>,
    pub payload: Option<String>,
}

/// Locator resolution evidence stored in the trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLocatorResolution {
    pub query: String,
    pub matched_target: Option<String>,
    pub candidate_targets: Vec<String>,
    pub failure_reason: Option<String>,
}

/// Assertion result stored in the trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceAssertionRecord {
    pub code: String,
    pub passed: bool,
    pub target: Option<String>,
    pub actual: Option<String>,
    pub expected: Option<String>,
}

/// Artifact metadata stored in the trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceArtifactRecord {
    pub kind: String,
    pub path: Option<String>,
    pub message: Option<String>,
}
