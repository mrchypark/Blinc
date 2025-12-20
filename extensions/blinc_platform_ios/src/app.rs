//! iOS App integration

/// iOS main entry point
#[cfg(target_os = "ios")]
pub fn ios_main() {
    // TODO: Initialize UIKit and Blinc runtime
}

/// Placeholder for non-iOS builds
#[cfg(not(target_os = "ios"))]
pub fn ios_main() {}
