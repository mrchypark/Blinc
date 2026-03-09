use serde::{Deserialize, Serialize};

/// Coarse lifecycle state for a platform runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Foreground,
    Background,
    Inactive,
    Destroyed,
}

/// Insets applied by system UI in logical points.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewportInsets {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

/// Window metrics captured from the active platform surface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct WindowMetrics {
    pub logical_width: f32,
    pub logical_height: f32,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
}

/// Snapshot of dynamic platform environment values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformEnvironmentSnapshot {
    pub lifecycle_state: LifecycleState,
    pub metrics: WindowMetrics,
    pub safe_area_insets: ViewportInsets,
    pub viewport_insets: ViewportInsets,
    pub is_dark_mode: bool,
}

impl Default for PlatformEnvironmentSnapshot {
    fn default() -> Self {
        Self {
            lifecycle_state: LifecycleState::Inactive,
            metrics: WindowMetrics::default(),
            safe_area_insets: ViewportInsets::default(),
            viewport_insets: ViewportInsets::default(),
            is_dark_mode: false,
        }
    }
}

/// Delta event emitted when a snapshot changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformEnvironmentChanged {
    pub previous: PlatformEnvironmentSnapshot,
    pub current: PlatformEnvironmentSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_defaults_to_inactive_zeroed_metrics() {
        let snapshot = PlatformEnvironmentSnapshot::default();
        assert_eq!(snapshot.lifecycle_state, LifecycleState::Inactive);
        assert_eq!(snapshot.metrics, WindowMetrics::default());
        assert_eq!(snapshot.safe_area_insets, ViewportInsets::default());
        assert!(!snapshot.is_dark_mode);
    }
}
