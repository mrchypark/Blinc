//! Bridge module for recorder integration.
//!
//! This module provides helpers for sending event data to blinc_recorder
//! via BlincContextState callbacks, avoiding circular dependencies.

use blinc_core::BlincContextState;
use std::any::Any;
use std::collections::HashMap;

/// Mouse button for recorder events
#[derive(Clone, Copy, Debug)]
pub enum RecorderMouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

impl From<crate::event_router::MouseButton> for RecorderMouseButton {
    fn from(btn: crate::event_router::MouseButton) -> Self {
        match btn {
            crate::event_router::MouseButton::Left => RecorderMouseButton::Left,
            crate::event_router::MouseButton::Right => RecorderMouseButton::Right,
            crate::event_router::MouseButton::Middle => RecorderMouseButton::Middle,
            crate::event_router::MouseButton::Back => RecorderMouseButton::Other(3),
            crate::event_router::MouseButton::Forward => RecorderMouseButton::Other(4),
            crate::event_router::MouseButton::Other(n) => RecorderMouseButton::Other(n as u8),
        }
    }
}

/// Modifier key state captured for recorder events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecorderModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Event data sent to recorder
#[derive(Clone, Debug)]
pub enum RecorderEventData {
    MouseDown {
        x: f32,
        y: f32,
        button: RecorderMouseButton,
        modifiers: RecorderModifiers,
        target_element: Option<String>,
    },
    MouseUp {
        x: f32,
        y: f32,
        button: RecorderMouseButton,
        modifiers: RecorderModifiers,
        target_element: Option<String>,
    },
    MouseMove {
        x: f32,
        y: f32,
        hover_element: Option<String>,
    },
    Click {
        x: f32,
        y: f32,
        button: RecorderMouseButton,
        modifiers: RecorderModifiers,
        target_element: Option<String>,
    },
    KeyDown {
        key_code: u32,
        modifiers: RecorderModifiers,
        is_repeat: bool,
        focused_element: Option<String>,
    },
    KeyUp {
        key_code: u32,
        modifiers: RecorderModifiers,
        focused_element: Option<String>,
    },
    TextInput {
        text: String,
        focused_element: Option<String>,
    },
    Scroll {
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
        target_element: Option<String>,
    },
    FocusChange {
        from: Option<String>,
        to: Option<String>,
    },
    HoverEnter {
        element_id: String,
        x: f32,
        y: f32,
    },
    HoverLeave {
        element_id: String,
        x: f32,
        y: f32,
    },
}

/// Send an event to the recorder if recording is enabled.
///
/// This is a no-op if BlincContextState is not initialized or no recorder callback is set.
pub fn record_event(event: RecorderEventData) {
    if let Some(ctx) = BlincContextState::try_get() {
        #[cfg(feature = "recorder")]
        maybe_install_recorder_hooks(ctx);

        if ctx.is_recording_events() {
            #[cfg(feature = "recorder")]
            {
                ctx.record_event(Box::new(to_recorded_event(event)) as Box<dyn Any + Send>);
                return;
            }
            #[cfg(not(feature = "recorder"))]
            ctx.record_event(Box::new(event) as Box<dyn Any + Send>);
        }
    }
}

/// Check if event recording is currently enabled.
pub fn is_recording() -> bool {
    BlincContextState::try_get()
        .map(|ctx| ctx.is_recording_events())
        .unwrap_or(false)
}

/// Check if snapshot recording is currently enabled.
pub fn is_recording_snapshots() -> bool {
    BlincContextState::try_get()
        .map(|ctx| ctx.is_recording_snapshots())
        .unwrap_or(false)
}

/// Check if update recording is currently enabled.
pub fn is_recording_updates() -> bool {
    BlincContextState::try_get()
        .map(|ctx| ctx.is_recording_updates())
        .unwrap_or(false)
}

/// Record an element update with category.
///
/// This is called when diff/stateful detects element changes.
pub fn record_update(element_id: &str, category: blinc_core::UpdateCategory) {
    if let Some(ctx) = BlincContextState::try_get() {
        #[cfg(feature = "recorder")]
        maybe_install_recorder_hooks(ctx);

        if ctx.is_recording_updates() {
            ctx.record_update(element_id, category);
        }
    }
}

/// Convert ChangeCategory from diff module to UpdateCategory for recording.
pub fn change_category_to_update(
    change: &crate::diff::ChangeCategory,
) -> blinc_core::UpdateCategory {
    if change.children {
        blinc_core::UpdateCategory::Structural
    } else if change.layout {
        blinc_core::UpdateCategory::Layout
    } else {
        blinc_core::UpdateCategory::Visual
    }
}

// ============================================================================
// Tree Snapshot Types
// ============================================================================

/// Rectangle for snapshot bounds.
#[derive(Clone, Debug)]
pub struct SnapshotRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl SnapshotRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Visual properties for element snapshots.
#[derive(Clone, Debug, Default)]
pub struct SnapshotVisualProps {
    pub background_color: Option<[f32; 4]>,
    pub border_color: Option<[f32; 4]>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub opacity: Option<f32>,
    pub transform: Option<[f32; 6]>,
    pub styles: HashMap<String, String>,
}

/// Snapshot of a single element.
#[derive(Clone, Debug)]
pub struct ElementSnapshotData {
    pub id: String,
    pub element_type: String,
    pub bounds: SnapshotRect,
    pub is_visible: bool,
    pub is_focused: bool,
    pub is_hovered: bool,
    pub is_interactive: bool,
    pub children: Vec<String>,
    pub parent: Option<String>,
    pub visual_props: Option<SnapshotVisualProps>,
    pub text_content: Option<String>,
}

/// Complete tree snapshot.
#[derive(Clone, Debug)]
pub struct TreeSnapshotData {
    pub elements: std::collections::HashMap<String, ElementSnapshotData>,
    pub root_id: Option<String>,
    pub focused_element: Option<String>,
    pub hovered_element: Option<String>,
    pub window_size: (u32, u32),
    pub scale_factor: f64,
}

impl TreeSnapshotData {
    /// Create an empty tree snapshot.
    pub fn new(window_size: (u32, u32), scale_factor: f64) -> Self {
        Self {
            elements: std::collections::HashMap::new(),
            root_id: None,
            focused_element: None,
            hovered_element: None,
            window_size,
            scale_factor,
        }
    }
}

/// Send a tree snapshot to the recorder if recording is enabled.
///
/// This is a no-op if BlincContextState is not initialized or no recorder callback is set.
pub fn record_snapshot(snapshot: TreeSnapshotData) {
    if let Some(ctx) = BlincContextState::try_get() {
        #[cfg(feature = "recorder")]
        maybe_install_recorder_hooks(ctx);

        if ctx.is_recording_snapshots() {
            #[cfg(feature = "recorder")]
            {
                ctx.record_snapshot(Box::new(to_tree_snapshot(snapshot)) as Box<dyn Any + Send>);
                return;
            }
            #[cfg(not(feature = "recorder"))]
            ctx.record_snapshot(Box::new(snapshot) as Box<dyn Any + Send>);
        }
    }
}

#[cfg(feature = "recorder")]
fn maybe_install_recorder_hooks(ctx: &BlincContextState) {
    if (ctx.is_recording_events() && ctx.is_recording_snapshots() && ctx.is_recording_updates())
        || blinc_recorder::get_recorder().is_none()
    {
        return;
    }

    blinc_recorder::install_hooks();
}

#[cfg(feature = "recorder")]
fn to_mouse_button(button: RecorderMouseButton) -> blinc_recorder::MouseButton {
    match button {
        RecorderMouseButton::Left => blinc_recorder::MouseButton::Left,
        RecorderMouseButton::Right => blinc_recorder::MouseButton::Right,
        RecorderMouseButton::Middle => blinc_recorder::MouseButton::Middle,
        RecorderMouseButton::Other(n) => blinc_recorder::MouseButton::Other(n),
    }
}

#[cfg(feature = "recorder")]
fn to_modifiers(modifiers: RecorderModifiers) -> blinc_recorder::Modifiers {
    blinc_recorder::Modifiers {
        shift: modifiers.shift,
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        meta: modifiers.meta,
    }
}

#[cfg(feature = "recorder")]
fn to_key(key_code: u32) -> blinc_recorder::Key {
    match key_code {
        8 => blinc_recorder::Key::Backspace,
        9 => blinc_recorder::Key::Tab,
        13 => blinc_recorder::Key::Enter,
        16 => blinc_recorder::Key::Shift,
        17 => blinc_recorder::Key::Control,
        18 => blinc_recorder::Key::Alt,
        20 => blinc_recorder::Key::CapsLock,
        27 => blinc_recorder::Key::Escape,
        32 => blinc_recorder::Key::Space,
        33 => blinc_recorder::Key::PageUp,
        34 => blinc_recorder::Key::PageDown,
        35 => blinc_recorder::Key::End,
        36 => blinc_recorder::Key::Home,
        37 => blinc_recorder::Key::Left,
        38 => blinc_recorder::Key::Up,
        39 => blinc_recorder::Key::Right,
        40 => blinc_recorder::Key::Down,
        45 => blinc_recorder::Key::Insert,
        46 => blinc_recorder::Key::Delete,
        48 => blinc_recorder::Key::Num0,
        49 => blinc_recorder::Key::Num1,
        50 => blinc_recorder::Key::Num2,
        51 => blinc_recorder::Key::Num3,
        52 => blinc_recorder::Key::Num4,
        53 => blinc_recorder::Key::Num5,
        54 => blinc_recorder::Key::Num6,
        55 => blinc_recorder::Key::Num7,
        56 => blinc_recorder::Key::Num8,
        57 => blinc_recorder::Key::Num9,
        65 => blinc_recorder::Key::A,
        66 => blinc_recorder::Key::B,
        67 => blinc_recorder::Key::C,
        68 => blinc_recorder::Key::D,
        69 => blinc_recorder::Key::E,
        70 => blinc_recorder::Key::F,
        71 => blinc_recorder::Key::G,
        72 => blinc_recorder::Key::H,
        73 => blinc_recorder::Key::I,
        74 => blinc_recorder::Key::J,
        75 => blinc_recorder::Key::K,
        76 => blinc_recorder::Key::L,
        77 => blinc_recorder::Key::M,
        78 => blinc_recorder::Key::N,
        79 => blinc_recorder::Key::O,
        80 => blinc_recorder::Key::P,
        81 => blinc_recorder::Key::Q,
        82 => blinc_recorder::Key::R,
        83 => blinc_recorder::Key::S,
        84 => blinc_recorder::Key::T,
        85 => blinc_recorder::Key::U,
        86 => blinc_recorder::Key::V,
        87 => blinc_recorder::Key::W,
        88 => blinc_recorder::Key::X,
        89 => blinc_recorder::Key::Y,
        90 => blinc_recorder::Key::Z,
        112 => blinc_recorder::Key::F1,
        113 => blinc_recorder::Key::F2,
        114 => blinc_recorder::Key::F3,
        115 => blinc_recorder::Key::F4,
        116 => blinc_recorder::Key::F5,
        117 => blinc_recorder::Key::F6,
        118 => blinc_recorder::Key::F7,
        119 => blinc_recorder::Key::F8,
        120 => blinc_recorder::Key::F9,
        121 => blinc_recorder::Key::F10,
        122 => blinc_recorder::Key::F11,
        123 => blinc_recorder::Key::F12,
        code => blinc_recorder::Key::Other(code),
    }
}

#[cfg(feature = "recorder")]
pub(crate) fn to_recorded_event(event: RecorderEventData) -> blinc_recorder::RecordedEvent {
    use blinc_recorder::{
        FocusChangeEvent, HoverEvent, KeyEvent, Modifiers, MouseEvent, MouseMoveEvent, Point,
        ScrollEvent, TextInputEvent, WindowResizeEvent,
    };

    match event {
        RecorderEventData::MouseDown {
            x,
            y,
            button,
            modifiers,
            target_element,
        } => blinc_recorder::RecordedEvent::MouseDown(MouseEvent {
            position: Point::new(x, y),
            button: to_mouse_button(button),
            modifiers: to_modifiers(modifiers),
            target_element,
        }),
        RecorderEventData::MouseUp {
            x,
            y,
            button,
            modifiers,
            target_element,
        } => blinc_recorder::RecordedEvent::MouseUp(MouseEvent {
            position: Point::new(x, y),
            button: to_mouse_button(button),
            modifiers: to_modifiers(modifiers),
            target_element,
        }),
        RecorderEventData::MouseMove {
            x,
            y,
            hover_element,
        } => blinc_recorder::RecordedEvent::MouseMove(MouseMoveEvent {
            position: Point::new(x, y),
            hover_element,
        }),
        RecorderEventData::Click {
            x,
            y,
            button,
            modifiers,
            target_element,
        } => blinc_recorder::RecordedEvent::Click(MouseEvent {
            position: Point::new(x, y),
            button: to_mouse_button(button),
            modifiers: to_modifiers(modifiers),
            target_element,
        }),
        RecorderEventData::KeyDown {
            key_code,
            modifiers,
            is_repeat,
            focused_element,
        } => blinc_recorder::RecordedEvent::KeyDown(KeyEvent {
            key: to_key(key_code),
            modifiers: to_modifiers(modifiers),
            is_repeat,
            focused_element,
        }),
        RecorderEventData::KeyUp {
            key_code,
            modifiers,
            focused_element,
        } => blinc_recorder::RecordedEvent::KeyUp(KeyEvent {
            key: to_key(key_code),
            modifiers: to_modifiers(modifiers),
            is_repeat: false,
            focused_element,
        }),
        RecorderEventData::TextInput {
            text,
            focused_element,
        } => blinc_recorder::RecordedEvent::TextInput(TextInputEvent {
            text,
            focused_element,
        }),
        RecorderEventData::Scroll {
            x,
            y,
            delta_x,
            delta_y,
            target_element,
        } => blinc_recorder::RecordedEvent::Scroll(ScrollEvent {
            position: Point::new(x, y),
            delta_x,
            delta_y,
            target_element,
        }),
        RecorderEventData::FocusChange { from, to } => {
            blinc_recorder::RecordedEvent::FocusChange(FocusChangeEvent { from, to })
        }
        RecorderEventData::HoverEnter { element_id, x, y } => {
            blinc_recorder::RecordedEvent::HoverEnter(HoverEvent {
                element_id,
                position: Point::new(x, y),
            })
        }
        RecorderEventData::HoverLeave { element_id, x, y } => {
            blinc_recorder::RecordedEvent::HoverLeave(HoverEvent {
                element_id,
                position: Point::new(x, y),
            })
        }
    }
}

#[cfg(feature = "recorder")]
pub(crate) fn to_tree_snapshot(snapshot: TreeSnapshotData) -> blinc_recorder::TreeSnapshot {
    let mut converted = blinc_recorder::TreeSnapshot::new(
        blinc_recorder::Timestamp::zero(),
        snapshot.window_size,
        snapshot.scale_factor,
    );
    converted.root_id = snapshot.root_id;
    converted.focused_element = snapshot.focused_element;
    converted.hovered_element = snapshot.hovered_element;
    converted.elements = snapshot
        .elements
        .into_iter()
        .map(|(id, element)| {
            let visual_props = element
                .visual_props
                .map(|props| blinc_recorder::VisualProps {
                    background_color: props.background_color,
                    border_color: props.border_color,
                    border_width: props.border_width,
                    border_radius: props.border_radius,
                    opacity: props.opacity,
                    transform: props.transform,
                    styles: props.styles,
                });

            (
                id,
                blinc_recorder::ElementSnapshot {
                    id: element.id,
                    element_type: element.element_type,
                    bounds: blinc_recorder::Rect::new(
                        element.bounds.x,
                        element.bounds.y,
                        element.bounds.width,
                        element.bounds.height,
                    ),
                    is_visible: element.is_visible,
                    is_focused: element.is_focused,
                    is_hovered: element.is_hovered,
                    is_interactive: element.is_interactive,
                    children: element.children,
                    parent: element.parent,
                    visual_props,
                    text_content: element.text_content,
                },
            )
        })
        .collect();
    converted
}

#[cfg(all(test, feature = "recorder"))]
mod tests {
    use super::*;

    #[test]
    fn converts_mouse_click_to_recorded_event() {
        let raw = RecorderEventData::Click {
            x: 10.0,
            y: 20.0,
            button: RecorderMouseButton::Left,
            modifiers: RecorderModifiers {
                shift: true,
                ctrl: false,
                alt: true,
                meta: false,
            },
            target_element: Some("node-1".to_string()),
        };

        let event = to_recorded_event(raw);
        match event {
            blinc_recorder::RecordedEvent::Click(click) => {
                assert_eq!(click.position.x, 10.0);
                assert_eq!(click.position.y, 20.0);
                assert_eq!(click.target_element.as_deref(), Some("node-1"));
                assert!(click.modifiers.shift);
                assert!(click.modifiers.alt);
                assert!(!click.modifiers.ctrl);
                assert!(!click.modifiers.meta);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn converts_tree_snapshot_to_recorder_snapshot() {
        let mut snapshot = TreeSnapshotData::new((800, 600), 2.0);
        snapshot.root_id = Some("root".to_string());
        snapshot.focused_element = Some("root".to_string());
        snapshot.hovered_element = Some("root".to_string());
        snapshot.elements.insert(
            "root".to_string(),
            ElementSnapshotData {
                id: "root".to_string(),
                element_type: "Div".to_string(),
                bounds: SnapshotRect::new(1.0, 2.0, 3.0, 4.0),
                is_visible: true,
                is_focused: true,
                is_hovered: true,
                is_interactive: true,
                children: vec![],
                parent: None,
                visual_props: Some(SnapshotVisualProps {
                    background_color: Some([1.0, 0.0, 0.0, 1.0]),
                    border_color: None,
                    border_width: Some(1.0),
                    border_radius: Some(2.0),
                    opacity: Some(0.5),
                    transform: Some([1.0, 0.0, 0.0, 1.0, 4.0, 8.0]),
                    styles: HashMap::from([("z-index".to_string(), "3".to_string())]),
                }),
                text_content: Some("hello".to_string()),
            },
        );

        let converted = to_tree_snapshot(snapshot);
        assert_eq!(converted.window_size, (800, 600));
        assert_eq!(converted.scale_factor, 2.0);
        assert_eq!(converted.root_id.as_deref(), Some("root"));
        assert!(converted.elements.contains_key("root"));
        let props = converted
            .elements
            .get("root")
            .and_then(|e| e.visual_props.as_ref())
            .expect("visual props");
        assert_eq!(props.transform, Some([1.0, 0.0, 0.0, 1.0, 4.0, 8.0]));
        assert_eq!(props.styles.get("z-index").map(String::as_str), Some("3"));
    }
}

/// Capture a tree snapshot from a RenderTree.
///
/// This walks the render tree and captures the current state of all elements.
/// The snapshot can then be sent to the recorder via `record_snapshot`.
pub fn capture_tree_snapshot(
    tree: &crate::renderer::RenderTree,
    focused_node: Option<crate::tree::LayoutNodeId>,
    hovered_nodes: &std::collections::HashSet<crate::tree::LayoutNodeId>,
    window_width: u32,
    window_height: u32,
) -> TreeSnapshotData {
    let scale_factor = tree.scale_factor() as f64;
    let mut snapshot = TreeSnapshotData::new((window_width, window_height), scale_factor);

    if let Some(root) = tree.root() {
        snapshot.root_id = Some(format!("{:?}", root));
        capture_node_recursive(tree, root, None, focused_node, hovered_nodes, &mut snapshot);
    }

    snapshot.focused_element = focused_node.map(|n| format!("{:?}", n));

    snapshot
}

/// Recursively capture a node and its children.
fn capture_node_recursive(
    tree: &crate::renderer::RenderTree,
    node: crate::tree::LayoutNodeId,
    parent: Option<crate::tree::LayoutNodeId>,
    focused_node: Option<crate::tree::LayoutNodeId>,
    hovered_nodes: &std::collections::HashSet<crate::tree::LayoutNodeId>,
    snapshot: &mut TreeSnapshotData,
) {
    let node_id_str = format!("{:?}", node);

    // Get bounds
    let bounds = tree
        .layout()
        .get_bounds(node, (0.0, 0.0))
        .map(|b| SnapshotRect::new(b.x, b.y, b.width, b.height))
        .unwrap_or_else(|| SnapshotRect::new(0.0, 0.0, 0.0, 0.0));

    // Get render node for element type and visual props
    let render_node = tree.get_render_node(node);
    let element_type = render_node
        .map(|n| match &n.element_type {
            crate::renderer::ElementType::Div => "Div".to_string(),
            crate::renderer::ElementType::Text(t) => format!("Text({})", t.content.len()),
            crate::renderer::ElementType::StyledText(_) => "StyledText".to_string(),
            crate::renderer::ElementType::Svg(_) => "Svg".to_string(),
            crate::renderer::ElementType::Image(_) => "Image".to_string(),
            crate::renderer::ElementType::Canvas(_) => "Canvas".to_string(),
        })
        .unwrap_or_else(|| "Unknown".to_string());

    // Extract visual props
    let visual_props = render_node.map(|n| {
        let props = &n.props;
        SnapshotVisualProps {
            background_color: props.background.as_ref().and_then(|brush| {
                if let blinc_core::Brush::Solid(c) = brush {
                    Some(c.to_array())
                } else {
                    None
                }
            }),
            border_color: props.border_color.map(|c| c.to_array()),
            border_width: if props.border_width > 0.0 {
                Some(props.border_width)
            } else {
                None
            },
            border_radius: Some(props.border_radius.top_left),
            opacity: Some(props.opacity),
            transform: extract_snapshot_transform(props.transform.as_ref()),
            styles: extract_snapshot_styles(props),
        }
    });

    // Extract text content if available
    let text_content = render_node.and_then(|n| match &n.element_type {
        crate::renderer::ElementType::Text(t) => Some(t.content.clone()),
        _ => None,
    });

    // Get children
    let children = tree.layout().children(node);
    let child_ids: Vec<String> = children.iter().map(|c| format!("{:?}", c)).collect();

    // Simplified check - a more thorough check would need handler registry access
    let is_interactive = render_node.is_some();

    let elem = ElementSnapshotData {
        id: node_id_str.clone(),
        element_type,
        bounds,
        is_visible: true, // Could check opacity/display later
        is_focused: focused_node == Some(node),
        is_hovered: hovered_nodes.contains(&node),
        is_interactive,
        children: child_ids,
        parent: parent.map(|p| format!("{:?}", p)),
        visual_props,
        text_content,
    };

    snapshot.elements.insert(node_id_str, elem);

    // Update hovered element in snapshot if this node is hovered
    if hovered_nodes.contains(&node) {
        snapshot.hovered_element = Some(format!("{:?}", node));
    }

    // Recurse into children
    for child in children {
        capture_node_recursive(
            tree,
            child,
            Some(node),
            focused_node,
            hovered_nodes,
            snapshot,
        );
    }
}

fn extract_snapshot_transform(transform: Option<&blinc_core::Transform>) -> Option<[f32; 6]> {
    match transform {
        Some(blinc_core::Transform::Affine2D(affine)) => Some(affine.elements),
        _ => None,
    }
}

fn extract_snapshot_styles(props: &crate::element::RenderProps) -> HashMap<String, String> {
    let mut styles = HashMap::new();

    if let Some(cursor) = props.cursor {
        styles.insert("cursor".to_string(), format!("{cursor:?}"));
    }
    if props.pointer_events_none {
        styles.insert("pointer-events".to_string(), "none".to_string());
    }
    if props.is_fixed {
        styles.insert("position".to_string(), "fixed".to_string());
    } else if props.is_sticky {
        styles.insert("position".to_string(), "sticky".to_string());
    }
    if let Some(top) = props.sticky_top {
        styles.insert("sticky-top".to_string(), top.to_string());
    }
    if let Some(bottom) = props.sticky_bottom {
        styles.insert("sticky-bottom".to_string(), bottom.to_string());
    }
    if props.z_index != 0 {
        styles.insert("z-index".to_string(), props.z_index.to_string());
    }
    if props.clips_content {
        styles.insert("overflow".to_string(), "clip".to_string());
    }
    if let Some([x, y]) = props.transform_origin {
        styles.insert("transform-origin".to_string(), format!("{x:.3},{y:.3}"));
    }
    if let Some(font_size) = props.font_size {
        styles.insert("font-size".to_string(), font_size.to_string());
    }

    styles
}
