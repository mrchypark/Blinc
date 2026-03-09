use blinc_core::native_bridge::native_call;

use crate::error::PlatformError;

/// Trigger selection feedback.
pub fn selection() -> Result<(), PlatformError> {
    let _: () = native_call("haptics", "selection", ())?;
    Ok(())
}

/// Trigger impact feedback using a platform-defined style string such as
/// `light`, `medium`, or `heavy`.
pub fn impact(style: &str) -> Result<(), PlatformError> {
    let _: () = native_call("haptics", "impact", (style.to_string(),))?;
    Ok(())
}
