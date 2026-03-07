mod support;

use blinc_platform::{FocusTraversalIntent, InputEvent, Key, KeyState};
use blinc_platform_desktop::input;
use winit::event::{ElementState, Ime};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};

#[test]
fn ime_notifications_convert_to_shared_composition_events() {
    let _display_available = support::requires_display();

    let enabled = input::convert_ime_event(&Ime::Enabled).expect("enabled composition event");
    let preview = input::convert_ime_event(&Ime::Preedit("한".into(), Some((0, 1))))
        .expect("preedit composition event");
    let committed =
        input::convert_ime_event(&Ime::Commit("한글".into())).expect("commit composition event");
    let disabled = input::convert_ime_event(&Ime::Disabled).expect("disabled composition event");

    assert!(matches!(enabled, InputEvent::CompositionStarted));
    assert!(matches!(
        preview,
        InputEvent::CompositionUpdated(update)
            if update.text == "한"
                && update.selection == Some(blinc_platform::ImeCompositionSelection::new(0, 1))
    ));
    assert!(matches!(
        committed,
        InputEvent::CompositionCommitted(text) if text == "한글"
    ));
    assert!(matches!(disabled, InputEvent::CompositionCancelled));
}

#[test]
fn plain_keypresses_still_convert_when_composition_is_inactive() {
    let event = input::convert_keyboard_event(
        &WinitKey::Character("a".into()),
        ElementState::Pressed,
        ModifiersState::empty(),
    );

    match event {
        InputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.key, Key::A);
            assert_eq!(keyboard.state, KeyState::Pressed);
        }
        other => panic!("expected keyboard input, got {other:?}"),
    }
}

#[test]
fn tab_focus_traversal_only_applies_without_shortcut_modifiers() {
    let plain_tab = input::convert_keyboard_event(
        &WinitKey::Named(NamedKey::Tab),
        ElementState::Pressed,
        ModifiersState::empty(),
    );
    let shift_tab = input::convert_keyboard_event(
        &WinitKey::Named(NamedKey::Tab),
        ElementState::Pressed,
        ModifiersState::SHIFT,
    );

    assert!(matches!(
        plain_tab,
        InputEvent::FocusTraversal(FocusTraversalIntent::Next)
    ));
    assert!(matches!(
        shift_tab,
        InputEvent::FocusTraversal(FocusTraversalIntent::Previous)
    ));

    for modifiers in [
        ModifiersState::CONTROL,
        ModifiersState::ALT,
        ModifiersState::SUPER,
        ModifiersState::CONTROL.union(ModifiersState::SHIFT),
    ] {
        let event = input::convert_keyboard_event(
            &WinitKey::Named(NamedKey::Tab),
            ElementState::Pressed,
            modifiers,
        );

        match event {
            InputEvent::Keyboard(keyboard) => {
                assert_eq!(keyboard.key, Key::Tab);
                assert_eq!(keyboard.state, KeyState::Pressed);
                assert_eq!(keyboard.modifiers.shift, modifiers.shift_key());
                assert_eq!(keyboard.modifiers.ctrl, modifiers.control_key());
                assert_eq!(keyboard.modifiers.alt, modifiers.alt_key());
                assert_eq!(keyboard.modifiers.meta, modifiers.super_key());
            }
            other => panic!("expected keyboard input for modified tab, got {other:?}"),
        }
    }
}

#[test]
fn empty_preedit_clear_after_commit_does_not_emit_cancellation() {
    let committed =
        input::convert_ime_event(&Ime::Commit("한글".into())).expect("commit composition event");
    let clear = input::convert_ime_event(&Ime::Preedit(String::new(), None));

    assert!(matches!(
        committed,
        InputEvent::CompositionCommitted(text) if text == "한글"
    ));
    assert!(matches!(
        clear,
        Some(InputEvent::CompositionUpdated(update))
            if update.text.is_empty() && update.selection.is_none()
    ));
}
