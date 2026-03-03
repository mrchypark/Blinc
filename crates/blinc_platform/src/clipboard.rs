use blinc_core::native_bridge::native_call;

use crate::error::PlatformError;

/// Copy text to the system clipboard.
pub fn copy(text: &str) -> Result<(), PlatformError> {
    let _: () = native_call("clipboard", "copy", (text.to_string(),))?;
    Ok(())
}

/// Read text from the system clipboard.
pub fn paste() -> Result<String, PlatformError> {
    let text: String = native_call("clipboard", "paste", ())?;
    Ok(text)
}

/// Returns true if clipboard currently has text content.
pub fn has_content() -> Result<bool, PlatformError> {
    let has_content: bool = native_call("clipboard", "has_content", ())?;
    Ok(has_content)
}

/// Clear clipboard contents.
pub fn clear() -> Result<(), PlatformError> {
    let _: () = native_call("clipboard", "clear", ())?;
    Ok(())
}
