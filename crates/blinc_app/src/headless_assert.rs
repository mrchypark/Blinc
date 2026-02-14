//! Assertion helpers for headless diagnostics goals.

use std::collections::HashMap;

/// Snapshot of app-observable state used for headless assertions.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticsSnapshot {
    pub elements: HashMap<String, DiagnosticsElement>,
}

/// Minimal element representation for diagnostics checks.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticsElement {
    pub text: Option<String>,
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
    let Some(text) = element.text.as_deref() else {
        return AssertionResult::Failed {
            code: "missing_text".to_string(),
            message: format!("{id}: text not available"),
        };
    };
    if text.contains(expected) {
        AssertionResult::Passed
    } else {
        AssertionResult::Failed {
            code: "text_mismatch".to_string(),
            message: format!("{id}: expected substring '{expected}', got '{text}'"),
        }
    }
}
