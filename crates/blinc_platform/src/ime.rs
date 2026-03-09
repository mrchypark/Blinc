use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImeCursorArea {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ImeCursorArea {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImeState {
    pub enabled: bool,
    pub cursor_area: Option<ImeCursorArea>,
}

fn ime_state_slot() -> &'static Mutex<ImeState> {
    static IME_STATE: OnceLock<Mutex<ImeState>> = OnceLock::new();
    IME_STATE.get_or_init(|| Mutex::new(ImeState::default()))
}

pub fn current_ime_state() -> ImeState {
    ime_state_slot()
        .lock()
        .map(|guard| *guard)
        .unwrap_or_default()
}

pub fn set_ime_state(state: ImeState) {
    if let Ok(mut guard) = ime_state_slot().lock() {
        *guard = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ime_state_round_trips() {
        let original = current_ime_state();
        let next = ImeState {
            enabled: true,
            cursor_area: Some(ImeCursorArea::new(10.0, 20.0, 30.0, 40.0)),
        };

        set_ime_state(next);
        assert_eq!(current_ime_state(), next);

        set_ime_state(original);
    }
}
