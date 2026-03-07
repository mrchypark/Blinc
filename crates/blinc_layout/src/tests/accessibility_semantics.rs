use blinc_platform::{AccessibilityRole, ImeCompositionSelection, ImeCompositionUpdate};
use blinc_theme::ThemeState;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard, Once};

use crate::accessibility::{export_accessibility_snapshot, focus_order};
use crate::div::div;
use crate::renderer::RenderTree;
use crate::stateful::{ButtonState, Stateful, TextFieldState};
use crate::widgets::{
    blur_all_text_inputs, button, set_focused_text_widget_composition, text_area, text_area_state,
    text_input, text_input_state,
};

fn ensure_theme() {
    static INIT: Once = Once::new();
    INIT.call_once(ThemeState::init_default);
}

fn accessibility_test_guard() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().expect("accessibility test lock")
}

#[test]
fn accessibility_semantics_preserve_preedit_text() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    blur_all_text_inputs();

    let input = text_input_state();
    {
        let mut data = input.lock().expect("input lock");
        data.value = "rust".to_string();
        data.cursor = 4;
        data.visual = TextFieldState::Focused;
    }
    crate::widgets::text_input::set_focused_text_input(&input);
    set_focused_text_widget_composition(Some(ImeCompositionUpdate::new(
        "한",
        Some(ImeCompositionSelection::new(0, 1)),
    )));

    let ui = div().child(text_input(&input));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(480.0, 120.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let input_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::TextInput)
        .expect("text input node");

    assert_eq!(input_node.value.as_deref(), Some("rust한"));

    blur_all_text_inputs();
}

#[test]
fn accessibility_semantics_export_roles() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    let button_state = Stateful::new(ButtonState::Idle).shared_state();
    let input = text_input_state();
    let area = text_area_state();

    let ui = div()
        .flex_col()
        .child(button(button_state, "Submit"))
        .child(text_input(&input))
        .child(text_area(&area));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(640.0, 320.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let roles = snapshot
        .nodes
        .iter()
        .map(|node| node.role)
        .collect::<Vec<_>>();

    assert!(roles.contains(&AccessibilityRole::Button));
    assert!(roles.contains(&AccessibilityRole::TextInput));
    assert!(roles.contains(&AccessibilityRole::TextArea));
}

#[test]
fn accessibility_semantics_focus_order() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    let button_state = Stateful::new(ButtonState::Idle).shared_state();
    let input = text_input_state();
    let area = text_area_state();

    let ui = div()
        .flex_col()
        .child(button(button_state, "Primary"))
        .child(text_input(&input))
        .child(text_area(&area));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(640.0, 320.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let ordered_roles = focus_order(&snapshot)
        .into_iter()
        .filter_map(|id| snapshot.nodes.iter().find(|node| node.id == id))
        .map(|node| node.role)
        .collect::<Vec<_>>();

    assert_eq!(
        ordered_roles,
        vec![
            AccessibilityRole::Button,
            AccessibilityRole::TextInput,
            AccessibilityRole::TextArea,
        ]
    );
}

#[test]
fn accessibility_semantics_snapshot_tracks_live_widget_state_without_rebuild() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    blur_all_text_inputs();

    let input = text_input_state();
    {
        let mut data = input.lock().expect("input lock");
        data.value = "a".to_string();
        data.cursor = 1;
        data.visual = TextFieldState::Focused;
    }

    let ui = div().child(text_input(&input));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(320.0, 120.0);

    {
        let mut data = input.lock().expect("input lock");
        data.value = "ab".to_string();
        data.cursor = 2;
    }

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let input_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::TextInput)
        .expect("text input node");

    assert_eq!(input_node.value.as_deref(), Some("ab"));

    blur_all_text_inputs();
}

#[test]
fn accessibility_semantics_nested_wrappers_keep_descendant_controls_reachable() {
    let _guard = accessibility_test_guard();
    ensure_theme();

    let input = text_input_state();
    let ui = div()
        .p(24.0)
        .child(div().p(16.0).child(div().p(12.0).child(text_input(&input))));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(400.0, 180.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let nodes_by_id = snapshot
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    let input_id = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::TextInput)
        .map(|node| node.id)
        .expect("text input node");

    let mut reachable = HashSet::new();
    let mut stack = vec![snapshot.root_id];
    while let Some(node_id) = stack.pop() {
        if !reachable.insert(node_id) {
            continue;
        }

        if let Some(node) = nodes_by_id.get(&node_id) {
            stack.extend(node.children.iter().copied());
        }
    }

    assert!(reachable.contains(&input_id));
}

#[test]
fn accessibility_semantics_ime_area_uses_absolute_widget_bounds() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    blur_all_text_inputs();

    let input = text_input_state();
    {
        let mut data = input.lock().expect("input lock");
        data.value = "hello".to_string();
        data.cursor = 5;
        data.visual = TextFieldState::Focused;
    }
    crate::widgets::text_input::set_focused_text_input(&input);

    let ui = div()
        .p(90.0)
        .child(div().pt(60.0).pl(40.0).child(text_input(&input)));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(640.0, 240.0);

    let ime_area = crate::widgets::focused_text_widget_ime_area().expect("ime area");

    assert!(
        ime_area.x >= 130.0,
        "ime x should include ancestor offsets: {ime_area:?}"
    );
    assert!(
        ime_area.y >= 150.0,
        "ime y should include ancestor offsets: {ime_area:?}"
    );

    blur_all_text_inputs();
}
