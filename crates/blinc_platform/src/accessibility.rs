//! Shared accessibility semantics contracts.

use std::sync::{Mutex, OnceLock};

/// Stable identifier for a semantic node.
pub type AccessibilityNodeId = u64;

/// Rectangle bounds for accessibility hit-testing and focus rings.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AccessibilityBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl AccessibilityBounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Platform-agnostic semantic roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityRole {
    Window,
    Group,
    Label,
    Button,
    Checkbox,
    TextInput,
    TextArea,
    Image,
}

/// Actions that a platform accessibility bridge can invoke on a semantic node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityAction {
    Focus,
    Press,
    Toggle,
    SetValue,
}

/// One semantic node exported from a UI tree.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityNode {
    pub id: AccessibilityNodeId,
    pub role: AccessibilityRole,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub bounds: Option<AccessibilityBounds>,
    pub focusable: bool,
    pub focused: bool,
    pub disabled: bool,
    pub actions: Vec<AccessibilityAction>,
    pub children: Vec<AccessibilityNodeId>,
}

impl AccessibilityNode {
    pub fn new(id: AccessibilityNodeId, role: AccessibilityRole) -> Self {
        Self {
            id,
            role,
            name: None,
            description: None,
            value: None,
            bounds: None,
            focusable: false,
            focused: false,
            disabled: false,
            actions: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_bounds(mut self, bounds: AccessibilityBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_actions(mut self, actions: Vec<AccessibilityAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn with_children(mut self, children: Vec<AccessibilityNodeId>) -> Self {
        self.children = children;
        self
    }
}

/// Snapshot of semantic nodes that a platform bridge can diff or publish.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AccessibilityTreeSnapshot {
    pub root_id: AccessibilityNodeId,
    pub nodes: Vec<AccessibilityNode>,
}

impl AccessibilityTreeSnapshot {
    pub fn new(root_id: AccessibilityNodeId, nodes: Vec<AccessibilityNode>) -> Self {
        Self { root_id, nodes }
    }
}

/// A platform accessibility action routed back into the app.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityActionRequest {
    pub target_id: AccessibilityNodeId,
    pub action: AccessibilityAction,
    pub value: Option<String>,
}

impl AccessibilityActionRequest {
    pub fn new(target_id: AccessibilityNodeId, action: AccessibilityAction) -> Self {
        Self {
            target_id,
            action,
            value: None,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

/// Last known accessibility sync status for the active runtime.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum AccessibilitySyncStatus {
    #[default]
    NotSynced,
    Synced,
    SnapshotOnly(String),
    Unsupported(String),
}

fn runtime_snapshot_store() -> &'static Mutex<Option<AccessibilityTreeSnapshot>> {
    static STORE: OnceLock<Mutex<Option<AccessibilityTreeSnapshot>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(None))
}

fn runtime_status_store() -> &'static Mutex<AccessibilitySyncStatus> {
    static STORE: OnceLock<Mutex<AccessibilitySyncStatus>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(AccessibilitySyncStatus::NotSynced))
}

pub fn update_platform_accessibility_snapshot(snapshot: AccessibilityTreeSnapshot) {
    if let Ok(mut current) = runtime_snapshot_store().lock() {
        *current = Some(snapshot);
    }
    if let Ok(mut status) = runtime_status_store().lock() {
        *status = AccessibilitySyncStatus::Synced;
    }
}

pub fn mark_accessibility_unsupported(reason: impl Into<String>) {
    let reason = reason.into();
    if let Ok(mut status) = runtime_status_store().lock() {
        *status = match &*status {
            AccessibilitySyncStatus::Synced | AccessibilitySyncStatus::SnapshotOnly(_) => {
                AccessibilitySyncStatus::SnapshotOnly(reason)
            }
            AccessibilitySyncStatus::NotSynced | AccessibilitySyncStatus::Unsupported(_) => {
                AccessibilitySyncStatus::Unsupported(reason)
            }
        };
    }
}

pub fn current_platform_accessibility_snapshot() -> Option<AccessibilityTreeSnapshot> {
    runtime_snapshot_store()
        .lock()
        .map(|snapshot| snapshot.clone())
        .unwrap_or(None)
}

pub fn current_accessibility_sync_status() -> AccessibilitySyncStatus {
    runtime_status_store()
        .lock()
        .map(|status| status.clone())
        .unwrap_or(AccessibilitySyncStatus::NotSynced)
}

pub fn reset_accessibility_runtime_state() {
    if let Ok(mut snapshot) = runtime_snapshot_store().lock() {
        *snapshot = None;
    }
    if let Ok(mut status) = runtime_status_store().lock() {
        *status = AccessibilitySyncStatus::NotSynced;
    }
}
