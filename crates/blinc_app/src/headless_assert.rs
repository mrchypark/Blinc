//! Assertion helpers for headless diagnostics goals.

use blinc_recorder::{ElementSnapshot, TreeSnapshot};
use std::collections::HashMap;

/// Snapshot of app-observable state used for headless assertions.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticsSnapshot {
    pub elements: HashMap<String, DiagnosticsElement>,
    tree: Option<TreeSnapshot>,
}

/// Minimal element representation for diagnostics checks.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticsElement {
    pub text: Option<String>,
}

impl DiagnosticsSnapshot {
    /// Build a diagnostics snapshot from a recorder tree snapshot.
    pub fn from_tree_snapshot(tree: TreeSnapshot) -> Self {
        let elements = tree
            .elements
            .iter()
            .map(|(id, element)| (id.clone(), DiagnosticsElement::from(element)))
            .collect();

        Self {
            elements,
            tree: Some(tree),
        }
    }

    /// Access the recorder-backed tree snapshot when available.
    pub fn tree(&self) -> Option<&TreeSnapshot> {
        self.tree.as_ref()
    }
}

impl From<TreeSnapshot> for DiagnosticsSnapshot {
    fn from(tree: TreeSnapshot) -> Self {
        Self::from_tree_snapshot(tree)
    }
}

impl From<&ElementSnapshot> for DiagnosticsElement {
    fn from(element: &ElementSnapshot) -> Self {
        Self {
            text: element.text_content.clone(),
        }
    }
}

/// Assertion result with structured failure details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertionResult {
    Passed,
    Failed { code: String, message: String },
}

pub fn evaluate_assert_exists(id: &str, snapshot: &DiagnosticsSnapshot) -> AssertionResult {
    if snapshot.elements.contains_key(id) {
        AssertionResult::Passed
    } else {
        AssertionResult::Failed {
            code: "missing_element".to_string(),
            message: format!("{id}: element not found"),
        }
    }
}

pub fn evaluate_assert_text_contains(
    id: &str,
    expected: &str,
    snapshot: &DiagnosticsSnapshot,
) -> AssertionResult {
    let Some(element) = snapshot.elements.get(id) else {
        return AssertionResult::Failed {
            code: "missing_element".to_string(),
            message: format!("{id}: element not found"),
        };
    };

    match element.text.as_deref() {
        Some(text) if text.contains(expected) => AssertionResult::Passed,
        Some(text) => AssertionResult::Failed {
            code: "text_mismatch".to_string(),
            message: format!("{id}: expected substring '{expected}', got '{text}'"),
        },
        None => AssertionResult::Failed {
            code: "missing_text".to_string(),
            message: format!("{id}: text not available"),
        },
    }
}
