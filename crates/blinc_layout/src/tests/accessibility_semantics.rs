use blinc_platform::{
    AccessibilityAction, AccessibilityRole, ImeCompositionSelection, ImeCompositionUpdate,
};
use blinc_theme::ThemeState;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard, Once};

use crate::accessibility::{export_accessibility_snapshot, focus_order};
use crate::div::div;
use crate::renderer::RenderTree;
use crate::stateful::{ButtonState, Stateful, TextFieldState};
use crate::widgets::{
    blur_all_text_inputs, button, button_with, scroll, set_focused_text_widget_composition,
    text_area, text_area_state, text_input, text_input_state,
};

fn ensure_theme() {
    static INIT: Once = Once::new();
    INIT.call_once(ThemeState::init_default);

    static CONTEXT_INIT: Once = Once::new();
    CONTEXT_INIT.call_once(|| {
        if !blinc_core::context_state::BlincContextState::is_initialized() {
            let reactive = Arc::new(Mutex::new(blinc_core::reactive::ReactiveGraph::new()));
            let hooks = Arc::new(Mutex::new(blinc_core::context_state::HookState::new()));
            let dirty = Arc::new(AtomicBool::new(false));
            blinc_core::context_state::BlincContextState::init(reactive, hooks, dirty);
        }
    });
}

fn accessibility_test_guard() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn bool_state(value: bool) -> blinc_core::State<bool> {
    let reactive = Arc::new(Mutex::new(blinc_core::reactive::ReactiveGraph::new()));
    let signal = reactive
        .lock()
        .expect("reactive graph lock")
        .create_signal(value);
    blinc_core::State::new(signal, reactive, Arc::new(AtomicBool::new(false)))
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
fn accessibility_semantics_masked_text_input_hides_plaintext() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    blur_all_text_inputs();

    let input = text_input_state();
    {
        let mut data = input.lock().expect("input lock");
        data.value = "secret".to_string();
        data.cursor = 6;
        data.visual = TextFieldState::Focused;
    }

    let ui = div().child(text_input(&input).masked(true));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(320.0, 120.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let input_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::TextInput)
        .expect("text input node");

    assert_eq!(input_node.value.as_deref(), Some("••••••"));
    assert_eq!(input_node.name, None);

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
fn accessibility_semantics_button_and_checkbox_export_actions() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    let button_state = Stateful::new(ButtonState::Idle).shared_state();
    let checked = bool_state(false);
    let checkbox = crate::widgets::checkbox::checkbox_labeled(&checked, "Enable");

    let ui = div()
        .flex_col()
        .child(button(button_state, "Submit"))
        .child(checkbox);
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(480.0, 220.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let button_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::Button)
        .expect("button node");
    let checkbox_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::Checkbox)
        .expect("checkbox node");

    assert!(button_node.actions.contains(&AccessibilityAction::Press));
    assert!(checkbox_node.actions.contains(&AccessibilityAction::Toggle));
}

#[test]
fn accessibility_semantics_disabled_controls_hide_invokable_actions() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    let button_state = Stateful::new(ButtonState::Disabled).shared_state();
    let checked = bool_state(true);

    let ui = div()
        .flex_col()
        .child(button(button_state, "Submit").disabled(true))
        .child(crate::widgets::checkbox::checkbox_labeled(&checked, "Enable").disabled(true));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(480.0, 220.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let button_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::Button)
        .expect("button node");
    let checkbox_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::Checkbox)
        .expect("checkbox node");

    assert!(button_node.disabled);
    assert!(checkbox_node.disabled);
    assert!(button_node.actions.is_empty());
    assert!(checkbox_node.actions.is_empty());
}

#[test]
fn accessibility_semantics_button_with_custom_content_infers_name() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    let button_state = Stateful::new(ButtonState::Idle).shared_state();

    let ui = div().child(button_with(button_state, |_state| {
        div().child(crate::text::text("Save"))
    }));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(320.0, 120.0);

    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let button_node = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::Button)
        .expect("button node");

    assert_eq!(button_node.name.as_deref(), Some("Save"));
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
fn accessibility_semantics_nested_wrappers_preserve_intermediate_groups() {
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
    let root = nodes_by_id.get(&snapshot.root_id).expect("root");

    assert_eq!(root.children.len(), 1, "expected synthetic wrapper group");
    let first_group = nodes_by_id.get(&root.children[0]).expect("first group");
    assert_eq!(first_group.role, AccessibilityRole::Group);
    assert_eq!(
        first_group.children.len(),
        1,
        "expected nested wrapper group"
    );
    let second_group = nodes_by_id
        .get(&first_group.children[0])
        .expect("second group");
    assert_eq!(second_group.role, AccessibilityRole::Group);
    assert_eq!(
        second_group.children.len(),
        1,
        "expected text input under second group"
    );
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

#[test]
fn accessibility_semantics_ime_area_and_bounds_follow_scroll_offsets() {
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

    let ui = scroll()
        .id("scroll-root")
        .w(280.0)
        .h(80.0)
        .vertical()
        .child(div().pt(120.0).child(text_input(&input)));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(280.0, 80.0);

    let before_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area before");
    let before_snapshot = export_accessibility_snapshot(&tree).expect("snapshot before");
    let before_input = before_snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::TextInput)
        .and_then(|node| node.bounds)
        .expect("input bounds before");

    let scroll_node = tree.query_by_id("scroll-root").expect("scroll node");
    tree.set_scroll_offset(scroll_node, 0.0, -48.0);
    tree.compute_layout(280.0, 80.0);

    let after_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area after");
    let after_snapshot = export_accessibility_snapshot(&tree).expect("snapshot after");
    let after_input = after_snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::TextInput)
        .and_then(|node| node.bounds)
        .expect("input bounds after");

    assert!(
        after_ime.y < before_ime.y - 40.0,
        "ime bounds should move with scroll offsets: before={before_ime:?} after={after_ime:?}"
    );
    assert!(
        after_input.y < before_input.y - 40.0,
        "accessibility bounds should move with scroll offsets: before={before_input:?} after={after_input:?}"
    );

    blur_all_text_inputs();
}

#[test]
fn accessibility_semantics_ime_area_updates_after_scroll_without_relayout() {
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

    let ui = scroll()
        .id("scroll-root")
        .w(280.0)
        .h(80.0)
        .vertical()
        .child(div().pt(120.0).child(text_input(&input)));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(280.0, 80.0);

    let before_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area before");

    let scroll_node = tree.query_by_id("scroll-root").expect("scroll node");
    tree.set_scroll_offset(scroll_node, 0.0, -48.0);
    tree.refresh_runtime_bounds();

    let after_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area after");

    assert!(
        after_ime.y < before_ime.y - 40.0,
        "ime bounds should update without relayout after scroll: before={before_ime:?} after={after_ime:?}"
    );

    blur_all_text_inputs();
}

#[test]
fn accessibility_semantics_fixed_children_ignore_scroll_offsets() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    blur_all_text_inputs();

    let fixed_input = text_input_state();
    let regular_input = text_input_state();
    {
        let mut data = fixed_input.lock().expect("fixed input lock");
        data.value = "hello".to_string();
        data.cursor = 5;
        data.visual = TextFieldState::Focused;
    }
    crate::widgets::text_input::set_focused_text_input(&fixed_input);

    {
        let mut data = regular_input.lock().expect("regular input lock");
        data.value = "world".to_string();
        data.cursor = 5;
    }

    let ui = scroll()
        .id("scroll-root")
        .w(280.0)
        .h(120.0)
        .vertical()
        .child(
            div()
                .flex_col()
                .h(400.0)
                .child(
                    div()
                        .fixed()
                        .top(12.0)
                        .left(16.0)
                        .child(text_input(&fixed_input).placeholder("Fixed")),
                )
                .child(
                    div()
                        .mt(140.0)
                        .child(text_input(&regular_input).placeholder("Regular")),
                ),
        );
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(280.0, 120.0);

    let before_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area before");
    let before_snapshot = export_accessibility_snapshot(&tree).expect("snapshot before");
    let before_fixed = before_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Fixed")
        })
        .and_then(|node| node.bounds)
        .expect("fixed input bounds before");
    let before_regular = before_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Regular")
        })
        .and_then(|node| node.bounds)
        .expect("regular input bounds before");

    let scroll_node = tree.query_by_id("scroll-root").expect("scroll node");
    tree.set_scroll_offset(scroll_node, 0.0, -48.0);
    tree.refresh_runtime_bounds();

    let after_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area after");
    let after_snapshot = export_accessibility_snapshot(&tree).expect("snapshot after");
    let after_fixed = after_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Fixed")
        })
        .and_then(|node| node.bounds)
        .expect("fixed input bounds after");
    let after_regular = after_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Regular")
        })
        .and_then(|node| node.bounds)
        .expect("regular input bounds after");

    assert!(
        (after_ime.y - before_ime.y).abs() < 1.0,
        "fixed IME area should stay pinned across scroll: before={before_ime:?} after={after_ime:?}"
    );
    assert!(
        (after_fixed.y - before_fixed.y).abs() < 1.0,
        "fixed accessibility bounds should stay pinned across scroll: before={before_fixed:?} after={after_fixed:?}"
    );
    assert!(
        after_regular.y < before_regular.y - 40.0,
        "non-fixed accessibility bounds should continue moving with scroll: before={before_regular:?} after={after_regular:?}"
    );

    blur_all_text_inputs();
}

#[test]
fn accessibility_semantics_sticky_children_clamp_after_scroll_threshold() {
    let _guard = accessibility_test_guard();
    ensure_theme();
    blur_all_text_inputs();

    let sticky_input = text_input_state();
    let regular_input = text_input_state();
    {
        let mut data = sticky_input.lock().expect("sticky input lock");
        data.value = "hello".to_string();
        data.cursor = 5;
        data.visual = TextFieldState::Focused;
    }
    crate::widgets::text_input::set_focused_text_input(&sticky_input);

    {
        let mut data = regular_input.lock().expect("regular input lock");
        data.value = "world".to_string();
        data.cursor = 5;
    }

    let ui = scroll()
        .id("scroll-root")
        .w(280.0)
        .h(96.0)
        .vertical()
        .child(
            div()
                .flex_col()
                .h(520.0)
                .child(
                    div().mt(120.0).child(
                        div()
                            .sticky(0.0)
                            .child(text_input(&sticky_input).placeholder("Sticky")),
                    ),
                )
                .child(
                    div()
                        .mt(120.0)
                        .child(text_input(&regular_input).placeholder("Regular")),
                ),
        );
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(280.0, 96.0);

    let before_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area before");
    let before_snapshot = export_accessibility_snapshot(&tree).expect("snapshot before");
    let before_sticky = before_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Sticky")
        })
        .and_then(|node| node.bounds)
        .expect("sticky input bounds before");
    let before_regular = before_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Regular")
        })
        .and_then(|node| node.bounds)
        .expect("regular input bounds before");

    let scroll_node = tree.query_by_id("scroll-root").expect("scroll node");
    tree.set_scroll_offset(scroll_node, 0.0, -160.0);
    tree.refresh_runtime_bounds();

    let after_ime = crate::widgets::focused_text_widget_ime_area().expect("ime area after");
    let after_snapshot = export_accessibility_snapshot(&tree).expect("snapshot after");
    let after_sticky = after_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Sticky")
        })
        .and_then(|node| node.bounds)
        .expect("sticky input bounds after");
    let after_regular = after_snapshot
        .nodes
        .iter()
        .find(|node| {
            node.role == AccessibilityRole::TextInput && node.name.as_deref() == Some("Regular")
        })
        .and_then(|node| node.bounds)
        .expect("regular input bounds after");

    assert!(
        (after_ime.y - before_ime.y).abs() < 1.0,
        "sticky IME area should stay pinned once the threshold is crossed: before={before_ime:?} after={after_ime:?}"
    );
    assert!(
        (after_sticky.y - before_sticky.y).abs() < 1.0,
        "sticky accessibility bounds should stay pinned once the threshold is crossed: before={before_sticky:?} after={after_sticky:?}"
    );
    assert!(
        after_regular.y < before_regular.y - 100.0,
        "non-sticky accessibility bounds should continue moving with scroll: before={before_regular:?} after={after_regular:?}"
    );

    blur_all_text_inputs();
}

#[test]
fn accessibility_semantics_ime_area_and_bounds_follow_transforms() {
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
        .p(40.0)
        .child(div().translate(28.0, 18.0).child(text_input(&input)));
    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(400.0, 180.0);

    let ime_area = crate::widgets::focused_text_widget_ime_area().expect("ime area");
    let snapshot = export_accessibility_snapshot(&tree).expect("snapshot");
    let input_bounds = snapshot
        .nodes
        .iter()
        .find(|node| node.role == AccessibilityRole::TextInput)
        .and_then(|node| node.bounds)
        .expect("input bounds");

    assert!(
        ime_area.x >= 68.0,
        "ime area should include transform offsets: {ime_area:?}"
    );
    assert!(
        ime_area.y >= 58.0,
        "ime area should include transform offsets: {ime_area:?}"
    );
    assert!(
        input_bounds.x >= 68.0,
        "accessibility bounds should include transform offsets: {input_bounds:?}"
    );
    assert!(
        input_bounds.y >= 58.0,
        "accessibility bounds should include transform offsets: {input_bounds:?}"
    );

    blur_all_text_inputs();
}
