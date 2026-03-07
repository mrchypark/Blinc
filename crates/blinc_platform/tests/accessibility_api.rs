use blinc_platform::{
    AccessibilityAction, AccessibilityBounds, AccessibilityNode, AccessibilityRole,
    AccessibilityTreeSnapshot, FocusTraversalIntent, ImeCompositionSelection, ImeCompositionUpdate,
    InputEvent,
};

#[test]
fn composition_lifecycle_events_capture_preview_and_commit_state() {
    let preview = ImeCompositionUpdate::new("한", Some(ImeCompositionSelection::new(0, 1)));

    let events = vec![
        InputEvent::CompositionStarted,
        InputEvent::CompositionUpdated(preview.clone()),
        InputEvent::CompositionCommitted("한글".to_string()),
        InputEvent::CompositionCancelled,
    ];

    assert!(matches!(events[0], InputEvent::CompositionStarted));
    assert!(matches!(
        &events[1],
        InputEvent::CompositionUpdated(update)
            if update.text == "한"
                && update.selection == Some(ImeCompositionSelection::new(0, 1))
    ));
    assert!(matches!(
        &events[2],
        InputEvent::CompositionCommitted(text) if text == "한글"
    ));
    assert!(matches!(events[3], InputEvent::CompositionCancelled));
}

#[test]
fn accessibility_snapshot_preserves_roles_metadata_and_actions() {
    let node = AccessibilityNode::new(7, AccessibilityRole::TextInput)
        .with_name("Search")
        .with_description("Type a query")
        .with_bounds(AccessibilityBounds::new(10.0, 20.0, 180.0, 36.0))
        .with_focusable(true)
        .with_actions(vec![
            AccessibilityAction::Focus,
            AccessibilityAction::SetValue,
        ]);

    let snapshot = AccessibilityTreeSnapshot::new(7, vec![node.clone()]);

    assert_eq!(snapshot.root_id, 7);
    assert_eq!(snapshot.nodes, vec![node.clone()]);
    assert_eq!(node.role, AccessibilityRole::TextInput);
    assert_eq!(node.name.as_deref(), Some("Search"));
    assert_eq!(node.description.as_deref(), Some("Type a query"));
    assert_eq!(
        node.bounds,
        Some(AccessibilityBounds::new(10.0, 20.0, 180.0, 36.0))
    );
    assert!(node.focusable);
    assert_eq!(
        node.actions,
        vec![AccessibilityAction::Focus, AccessibilityAction::SetValue]
    );
}

#[test]
fn focus_traversal_intents_stay_platform_agnostic() {
    let next = InputEvent::FocusTraversal(FocusTraversalIntent::Next);
    let previous = InputEvent::FocusTraversal(FocusTraversalIntent::Previous);

    assert!(matches!(
        next,
        InputEvent::FocusTraversal(FocusTraversalIntent::Next)
    ));
    assert!(matches!(
        previous,
        InputEvent::FocusTraversal(FocusTraversalIntent::Previous)
    ));
}
