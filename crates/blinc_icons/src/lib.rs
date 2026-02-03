//! # Blinc Icons
//!
//! Lucide-based icon library for Blinc UI framework.
//!
//! All ~1000+ Lucide icons are available as `pub const` values.
//! Unused icons are automatically eliminated by Rust's Dead Code Elimination (DCE).
//!
//! ## Usage
//!
//! ```ignore
//! use blinc_icons::{icons, to_svg};
//!
//! // Direct const access (DCE-friendly)
//! let svg = to_svg(icons::ARROW_RIGHT, 24.0);
//!
//! // Or use cn::icon() which wraps this
//! cn::icon(icons::CHECK).size(IconSize::Medium)
//! ```

// Generated icon constants module
// NOTE: `icons.rs` is generated and intentionally not rustfmt'd.
#[rustfmt::skip]
pub mod icons;

/// Default Lucide viewBox (all icons are 24x24)
pub const VIEW_BOX: (f32, f32, f32, f32) = (0.0, 0.0, 24.0, 24.0);

/// Default stroke width for Lucide icons
pub const STROKE_WIDTH: f32 = 2.0;

/// Generate a complete SVG string from icon path data
///
/// # Arguments
/// * `path_data` - The SVG path data (inner content of the SVG)
/// * `size` - The width and height of the SVG in pixels
///
/// # Example
/// ```ignore
/// let svg = to_svg(icons::CHECK, 24.0);
/// ```
pub fn to_svg(path_data: &str, size: f32) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{path_data}</svg>"#
    )
}

/// Generate SVG with custom stroke width
pub fn to_svg_with_stroke(path_data: &str, size: f32, stroke_width: f32) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="{stroke_width}" stroke-linecap="round" stroke-linejoin="round">{path_data}</svg>"#
    )
}

/// Generate SVG with custom color (for non-currentColor usage)
pub fn to_svg_colored(path_data: &str, size: f32, color: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{path_data}</svg>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_svg() {
        let svg = to_svg(icons::CHECK, 24.0);
        assert!(svg.contains("viewBox=\"0 0 24 24\""));
        assert!(svg.contains("width=\"24\""));
        assert!(svg.contains("stroke-width=\"2\""));
    }

    #[test]
    fn test_to_svg_with_stroke() {
        let svg = to_svg_with_stroke(icons::CHECK, 16.0, 1.5);
        assert!(svg.contains("width=\"16\""));
        assert!(svg.contains("stroke-width=\"1.5\""));
    }
}
