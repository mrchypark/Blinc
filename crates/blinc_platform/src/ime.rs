//! Shared desktop IME state.

use std::sync::{Mutex, OnceLock};

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

/// Requested IME state from the currently focused text control.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImeState {
    pub enabled: bool,
    pub cursor_area: Option<ImeCursorArea>,
}

fn global_ime_state() -> &'static Mutex<ImeState> {
    static STATE: OnceLock<Mutex<ImeState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(ImeState::default()))
}

pub fn current_ime_state() -> ImeState {
    global_ime_state()
        .lock()
        .map(|state| *state)
        .unwrap_or_default()
}

pub fn set_ime_state(state: ImeState) {
    if let Ok(mut current) = global_ime_state().lock() {
        *current = state;
    }
}
