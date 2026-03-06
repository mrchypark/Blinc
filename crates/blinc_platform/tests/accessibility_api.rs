use blinc_platform::{
    AccessibilityAction, AccessibilityActionRequest, AccessibilityBounds, AccessibilityNode,
    AccessibilityNodeId, AccessibilityRole, AccessibilityTreeSnapshot, CompositionEvent,
    CompositionUpdate, Event, FocusTraversalIntent, InputEvent, SelectionRange,
};

#[test]
fn composition_lifecycle_events_roundtrip() {
    let preview = CompositionUpdate {
        text: "ga".to_string(),
        selection: Some(SelectionRange { start: 1, end: 2 }),
    };
    let events = vec![
        InputEvent::Composition(CompositionEvent::Started),
        InputEvent::Composition(CompositionEvent::Updated(preview.clone())),
        InputEvent::Composition(CompositionEvent::Committed("각".to_string())),
        InputEvent::Composition(CompositionEvent::Cancelled),
    ];

    assert!(matches!(
        &events[0],
        InputEvent::Composition(CompositionEvent::Started)
    ));
    assert!(matches!(
        &events[1],
        InputEvent::Composition(CompositionEvent::Updated(update))
            if update.text == preview.text && update.selection == preview.selection
    ));
    assert!(matches!(
        &events[2],
        InputEvent::Composition(CompositionEvent::Committed(text)) if text == "각"
    ));
    assert!(matches!(
        &events[3],
        InputEvent::Composition(CompositionEvent::Cancelled)
    ));
}

#[test]
fn accessibility_nodes_capture_role_metadata_and_bounds() {
    let node = AccessibilityNode {
        id: AccessibilityNodeId(7),
        role: AccessibilityRole::TextInput,
        name: Some("Search".to_string()),
        description: Some("Filter the current list".to_string()),
        bounds: AccessibilityBounds {
            x: 12.0,
            y: 24.0,
            width: 240.0,
            height: 36.0,
        },
        focusable: true,
        focused: true,
        disabled: false,
        value: Some("rust".to_string()),
        children: vec![],
    };
    let snapshot = AccessibilityTreeSnapshot {
        root: node.id,
        nodes: vec![node.clone()],
    };

    assert_eq!(snapshot.root, AccessibilityNodeId(7));
    assert_eq!(snapshot.nodes.len(), 1);
    assert_eq!(snapshot.nodes[0].role, AccessibilityRole::TextInput);
    assert_eq!(snapshot.nodes[0].name.as_deref(), Some("Search"));
    assert_eq!(
        snapshot.nodes[0].description.as_deref(),
        Some("Filter the current list")
    );
    assert_eq!(
        snapshot.nodes[0].bounds,
        AccessibilityBounds {
            x: 12.0,
            y: 24.0,
            width: 240.0,
            height: 36.0,
        }
    );
    assert!(snapshot.nodes[0].focusable);
    assert!(snapshot.nodes[0].focused);
}

#[test]
fn focus_traversal_intents_stay_in_shared_accessibility_events() {
    let event = Event::AccessibilityAction(AccessibilityActionRequest {
        target: None,
        action: AccessibilityAction::FocusTraversal(FocusTraversalIntent::Next),
    });

    assert!(matches!(
        event,
        Event::AccessibilityAction(AccessibilityActionRequest {
            target: None,
            action: AccessibilityAction::FocusTraversal(FocusTraversalIntent::Next),
        })
    ));
}
