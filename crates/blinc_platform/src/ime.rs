//! Shared desktop IME state.

use std::sync::{Mutex, OnceLock};

use crate::ImeCompositionSelection;

/// Cursor anchor area used to position candidate/popup windows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImeCursorArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ImeCursorArea {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Stable identifier for the currently focused text input session.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextInputSessionId(String);

impl TextInputSessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether the platform keyboard/IME should be visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImeVisibility {
    #[default]
    Hidden,
    Visible,
}

/// Selection range for the focused text input in UTF-8 codepoint offsets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SelectionRange {
    pub start: usize,
    pub end: usize,
}

impl SelectionRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Runtime IME request emitted by the focused text input control.
#[derive(Clone, Debug, PartialEq)]
pub struct ImeRequest {
    pub session: TextInputSessionId,
    pub visibility: ImeVisibility,
    pub cursor_area: Option<ImeCursorArea>,
    pub selection: Option<SelectionRange>,
    pub composition: Option<ImeCompositionSelection>,
}

impl ImeRequest {
    pub fn new(session: TextInputSessionId) -> Self {
        Self {
            session,
            visibility: ImeVisibility::Hidden,
            cursor_area: None,
            selection: None,
            composition: None,
        }
    }

    pub fn with_visibility(mut self, visibility: ImeVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_cursor_area(mut self, cursor_area: ImeCursorArea) -> Self {
        self.cursor_area = Some(cursor_area);
        self
    }

    pub fn with_selection(mut self, selection: SelectionRange) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn with_composition(mut self, composition: Option<ImeCompositionSelection>) -> Self {
        self.composition = composition;
        self
    }
}

/// Requested IME state from the currently focused text control.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImeState {
    pub enabled: bool,
    pub cursor_area: Option<ImeCursorArea>,
    pub request: Option<ImeRequest>,
}

impl ImeState {
    pub fn with_request(mut self, request: Option<ImeRequest>) -> Self {
        self.enabled = request.is_some();
        self.cursor_area = request.as_ref().and_then(|request| request.cursor_area);
        self.request = request;
        self
    }
}

fn global_ime_state() -> &'static Mutex<ImeState> {
    static STATE: OnceLock<Mutex<ImeState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(ImeState::default()))
}

pub fn current_ime_state() -> ImeState {
    global_ime_state()
        .lock()
        .map(|state| state.clone())
        .unwrap_or_default()
}

pub fn set_ime_state(state: ImeState) {
    if let Ok(mut current) = global_ime_state().lock() {
        *current = state;
    }
}
