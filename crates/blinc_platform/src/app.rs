use blinc_core::native_bridge::native_call;

use crate::error::PlatformError;

/// Open a URL using the platform handler.
pub fn open_url(url: &str) -> Result<bool, PlatformError> {
    Ok(native_call("app", "open_url", (url.to_string(),))?)
}

/// Share a text payload using the platform handler.
pub fn share_text(text: &str) -> Result<(), PlatformError> {
    let _: () = native_call("app", "share_text", (text.to_string(),))?;
    Ok(())
}
