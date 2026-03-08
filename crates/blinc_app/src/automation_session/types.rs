use std::error::Error;
use std::fmt::{Display, Formatter};

use blinc_layout::selector::SemanticLocator;
use blinc_recorder::RecordingExport;

use crate::headless_report::HeadlessReport;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutomationLocator {
    Id(String),
    Semantic(SemanticLocator),
}

impl AutomationLocator {
    pub fn id(id: impl Into<String>) -> Self {
        Self::Id(id.into())
    }

    pub fn semantic(locator: SemanticLocator) -> Self {
        Self::Semantic(locator)
    }

    pub fn describe(&self) -> String {
        match self {
            AutomationLocator::Id(id) => format!("id={id:?}"),
            AutomationLocator::Semantic(locator) => locator.describe(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutomationFailure {
    pub code: String,
    pub message: String,
    pub target: Option<String>,
    pub trace_sequence: Option<u64>,
}

impl Display for AutomationFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for AutomationFailure {}

#[derive(Clone, Debug)]
pub struct AutomationRun {
    pub report: HeadlessReport,
    pub export: RecordingExport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationRuntimeMode {
    Headless,
    DesktopHarness,
}

pub(super) struct ParsedKey {
    pub(super) key_code: u32,
    pub(super) modifiers: u8,
    pub(super) text: Option<char>,
}

pub(super) fn parse_key(input: &str) -> Option<ParsedKey> {
    let normalized = input.trim();
    match normalized {
        "Enter" => Some(ParsedKey {
            key_code: 13,
            modifiers: 0,
            text: None,
        }),
        "Tab" => Some(ParsedKey {
            key_code: 9,
            modifiers: 0,
            text: None,
        }),
        "Escape" => Some(ParsedKey {
            key_code: 27,
            modifiers: 0,
            text: None,
        }),
        "Backspace" => Some(ParsedKey {
            key_code: 8,
            modifiers: 0,
            text: None,
        }),
        "Delete" => Some(ParsedKey {
            key_code: 127,
            modifiers: 0,
            text: None,
        }),
        "ArrowLeft" => Some(ParsedKey {
            key_code: 37,
            modifiers: 0,
            text: None,
        }),
        "ArrowRight" => Some(ParsedKey {
            key_code: 39,
            modifiers: 0,
            text: None,
        }),
        "ArrowUp" => Some(ParsedKey {
            key_code: 38,
            modifiers: 0,
            text: None,
        }),
        "ArrowDown" => Some(ParsedKey {
            key_code: 40,
            modifiers: 0,
            text: None,
        }),
        _ if normalized.chars().count() == 1 => Some(ParsedKey {
            key_code: normalized.chars().next()? as u32,
            modifiers: 0,
            text: normalized.chars().next(),
        }),
        _ => None,
    }
}

pub(super) fn select_all_modifiers() -> u8 {
    if cfg!(target_os = "macos") {
        0b1000
    } else {
        0b0010
    }
}
