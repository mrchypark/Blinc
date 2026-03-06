//! Desktop accessibility boundary.

use std::sync::{Mutex, OnceLock};

use blinc_platform::AccessibilityTreeSnapshot;

fn snapshot_store() -> &'static Mutex<Option<AccessibilityTreeSnapshot>> {
    static STORE: OnceLock<Mutex<Option<AccessibilityTreeSnapshot>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(None))
}

pub fn update_accessibility_snapshot(snapshot: AccessibilityTreeSnapshot) {
    if let Ok(mut current) = snapshot_store().lock() {
        *current = Some(snapshot);
    }
}

pub fn current_accessibility_snapshot() -> Option<AccessibilityTreeSnapshot> {
    snapshot_store()
        .lock()
        .map(|snapshot| snapshot.clone())
        .unwrap_or(None)
}

#[cfg(test)]
mod tests {
    use super::{current_accessibility_snapshot, update_accessibility_snapshot};
    use blinc_platform::{AccessibilityNode, AccessibilityRole, AccessibilityTreeSnapshot};

    #[test]
    fn snapshot_store_roundtrips_latest_value() {
        let snapshot =
            AccessibilityTreeSnapshot::new(1, vec![AccessibilityNode::new(1, AccessibilityRole::Window)]);
        update_accessibility_snapshot(snapshot.clone());
        assert_eq!(current_accessibility_snapshot(), Some(snapshot));
    }
}
