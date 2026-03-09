use blinc_platform::{
    current_accessibility_sync_status, current_platform_accessibility_snapshot,
    mark_accessibility_unsupported, reset_accessibility_runtime_state,
    update_platform_accessibility_snapshot, AccessibilityNode, AccessibilityRole,
    AccessibilitySyncStatus, AccessibilityTreeSnapshot,
};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn runtime_state_guard() -> MutexGuard<'static, ()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("accessibility runtime test mutex poisoned")
}

#[test]
fn platform_accessibility_snapshot_round_trips_latest_value() {
    let _guard = runtime_state_guard();
    reset_accessibility_runtime_state();
    let snapshot = AccessibilityTreeSnapshot::new(
        11,
        vec![AccessibilityNode::new(11, AccessibilityRole::Window)],
    );

    update_platform_accessibility_snapshot(snapshot.clone());

    assert_eq!(current_platform_accessibility_snapshot(), Some(snapshot));
    assert_eq!(
        current_accessibility_sync_status(),
        AccessibilitySyncStatus::Synced
    );
}

#[test]
fn unsupported_accessibility_sync_records_diagnostic() {
    let _guard = runtime_state_guard();
    reset_accessibility_runtime_state();
    mark_accessibility_unsupported("mobile accessibility bridge is not wired");

    assert_eq!(
        current_accessibility_sync_status(),
        AccessibilitySyncStatus::Unsupported(
            "mobile accessibility bridge is not wired".to_string()
        )
    );
}

#[test]
fn unsupported_bridge_after_snapshot_reports_snapshot_only() {
    let _guard = runtime_state_guard();
    reset_accessibility_runtime_state();
    let snapshot = AccessibilityTreeSnapshot::new(
        21,
        vec![AccessibilityNode::new(21, AccessibilityRole::Window)],
    );

    update_platform_accessibility_snapshot(snapshot.clone());
    mark_accessibility_unsupported("bridge deferred");

    assert_eq!(current_platform_accessibility_snapshot(), Some(snapshot));
    assert_eq!(
        current_accessibility_sync_status(),
        AccessibilitySyncStatus::SnapshotOnly("bridge deferred".to_string())
    );
}
