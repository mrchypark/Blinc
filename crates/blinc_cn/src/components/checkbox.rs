//! Checkbox component for boolean selection
//!
//! A themed checkbox component with checked, unchecked, and hover states.
//! Uses motion animations for smooth state transitions.
//!
//! # Example
//!
//! ```ignore
//! use blinc_cn::prelude::*;
//!
//! fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
//!     // Create checkbox state from context
//!     let checked = ctx.use_state_for("my_checkbox", false);
//!
//!     cn::checkbox(&checked)
//!         .label("Accept terms")
//!         .on_change(|is_checked| println!("Checked: {}", is_checked))
//! }
//!
//! // Pre-checked
//! let checked = ctx.use_state_for("agree", true);
//! cn::checkbox(&checked)
//!
//! // Disabled
//! cn::checkbox(&checked)
//!     .disabled(true)
//!
//! // Custom colors
//! cn::checkbox(&checked)
//!     .checked_color(Color::GREEN)
//!     .border_color(Color::GRAY)
//! ```

use blinc_core::{Color, State};
use blinc_layout::div::ElementTypeId;
use blinc_layout::element::RenderProps;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_theme::{ColorToken, RadiusToken, ThemeState};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// SVG checkmark path - simple checkmark that fits in a 16x16 viewBox
const CHECKMARK_SVG: &str = r#"<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M3 8L6.5 11.5L13 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

/// Checkbox size variants
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckboxSize {
    /// Small checkbox (14px)
    Small,
    /// Medium checkbox (18px)
    #[default]
    Medium,
    /// Large checkbox (22px)
    Large,
}

impl CheckboxSize {
    fn size(&self) -> f32 {
        match self {
            CheckboxSize::Small => 14.0,
            CheckboxSize::Medium => 18.0,
            CheckboxSize::Large => 22.0,
        }
    }

    fn border_width(&self) -> f32 {
        match self {
            CheckboxSize::Small => 1.5,
            CheckboxSize::Medium => 2.0,
            CheckboxSize::Large => 2.0,
        }
    }

    fn checkmark_size(&self) -> f32 {
        match self {
            CheckboxSize::Small => 10.0,
            CheckboxSize::Medium => 12.0,
            CheckboxSize::Large => 16.0,
        }
    }

    fn corner_radius(&self, theme: &ThemeState) -> f32 {
        match self {
            CheckboxSize::Small => theme.radius(RadiusToken::Sm) * 0.75,
            CheckboxSize::Medium => theme.radius(RadiusToken::Sm),
            CheckboxSize::Large => theme.radius(RadiusToken::Sm),
        }
    }
}

/// Checkbox component
///
/// A toggle checkbox with hover and press feedback.
/// Uses State<bool> from context for reactive state management.
pub struct Checkbox {
    inner: Stateful<ButtonState>,
    checked_state: State<bool>,
    size: CheckboxSize,
    label: Option<String>,
    disabled: bool,
    // Colors
    checked_color: Option<Color>,
    unchecked_bg: Option<Color>,
    border_color: Option<Color>,
    hover_border_color: Option<Color>,
    check_color: Option<Color>,
    // Callback
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl Checkbox {
    /// Create a new checkbox with state from context
    ///
    /// # Example
    /// ```ignore
    /// let checked = ctx.use_state_for("my_checkbox", false);
    /// cn::checkbox(&checked)
    /// ```
    pub fn new(checked_state: &State<bool>) -> Self {
        let inner = Stateful::new(ButtonState::Idle);

        Self {
            inner,
            checked_state: checked_state.clone(),
            size: CheckboxSize::default(),
            label: None,
            disabled: false,
            checked_color: None,
            unchecked_bg: None,
            border_color: None,
            hover_border_color: None,
            check_color: None,
            on_change: None,
        }
    }

    /// Set the checkbox size
    pub fn size(mut self, size: CheckboxSize) -> Self {
        self.size = size;
        self
    }

    /// Add a label to the checkbox
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the background color when checked
    pub fn checked_color(mut self, color: impl Into<Color>) -> Self {
        self.checked_color = Some(color.into());
        self
    }

    /// Set the background color when unchecked
    pub fn unchecked_bg(mut self, color: impl Into<Color>) -> Self {
        self.unchecked_bg = Some(color.into());
        self
    }

    /// Set the border color
    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    /// Set the hover border color
    pub fn hover_border_color(mut self, color: impl Into<Color>) -> Self {
        self.hover_border_color = Some(color.into());
        self
    }

    /// Set the checkmark color
    pub fn check_color(mut self, color: impl Into<Color>) -> Self {
        self.check_color = Some(color.into());
        self
    }

    /// Set the change callback
    ///
    /// Called when the checkbox is toggled, with the new checked state.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }

    /// Build the checkbox element
    fn build_checkbox(&self) -> Stateful<ButtonState> {
        let theme = ThemeState::get();
        let box_size = self.size.size();
        let border_width = self.size.border_width();
        let checkmark_size = self.size.checkmark_size();
        let radius = self.size.corner_radius(&theme);

        // Get colors
        let checked_bg = self
            .checked_color
            .unwrap_or_else(|| theme.color(ColorToken::Primary));
        let unchecked_bg = self
            .unchecked_bg
            .unwrap_or_else(|| theme.color(ColorToken::InputBg));
        let border = self
            .border_color
            .unwrap_or_else(|| theme.color(ColorToken::Border));
        let hover_border = self
            .hover_border_color
            .unwrap_or_else(|| theme.color(ColorToken::BorderHover));
        let check_mark_color = self
            .check_color
            .unwrap_or_else(|| theme.color(ColorToken::TextInverse));

        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let checked_state = self.checked_state.clone();
        let checked_state_for_click = self.checked_state.clone();

        let mut checkbox = Stateful::new(ButtonState::Idle)
            .w(box_size)
            .h(box_size)
            .rounded(radius)
            .cursor_pointer()
            .items_center()
            .justify_center()
            // Subscribe to the checked state signal for reactive updates
            .deps(&[checked_state.signal_id()]);

        if disabled {
            checkbox = checkbox.opacity(0.5);
        }

        // State callback for visual changes with motion-like transitions
        checkbox = checkbox.on_state(move |state: &ButtonState, container: &mut Div| {
            let is_checked = checked_state.get();
            let is_hovered = matches!(state, ButtonState::Hovered | ButtonState::Pressed);

            // Background and border with smooth color transitions
            let bg = if is_checked { checked_bg } else { unchecked_bg };
            let current_border = if is_hovered && !disabled {
                hover_border
            } else {
                border
            };

            // Apply scale effect on hover for subtle motion feedback
            let scale = if is_hovered && !disabled { 1.05 } else { 1.0 };

            // Build visual update - use merge to preserve existing properties
            let mut visual = div()
                .bg(bg)
                .border(border_width, current_border)
                .transform(blinc_core::Transform::scale(scale, scale));

            // Add checkmark if checked using SVG
            if is_checked {
                visual = visual.child(
                    svg(CHECKMARK_SVG)
                        .size(checkmark_size, checkmark_size)
                        .tint(check_mark_color),
                );
            }

            container.merge(visual);
        });

        // Add click handler to toggle the state
        checkbox = checkbox.on_click(move |_| {
            let current = checked_state_for_click.get();
            let new_value = !current;
            checked_state_for_click.set(new_value);

            if let Some(ref callback) = on_change {
                callback(new_value);
            }
        });

        checkbox
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        // Note: This default requires State<bool> which needs context
        // Prefer using checkbox(&state) constructor
        panic!("Checkbox requires State<bool> from context. Use checkbox(&state) instead.")
    }
}

impl Deref for Checkbox {
    type Target = Stateful<ButtonState>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Checkbox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for Checkbox {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        let checkbox = self.build_checkbox();

        // If there's a label, wrap in a row
        if let Some(ref label_text) = self.label {
            let theme = ThemeState::get();
            let label_color = if self.disabled {
                theme.color(ColorToken::TextTertiary)
            } else {
                theme.color(ColorToken::TextPrimary)
            };

            div()
                .flex_row()
                .gap(8.0)
                .items_center()
                .cursor_pointer()
                .child(checkbox)
                .child(text(label_text).size(14.0).color(label_color))
                .build(tree)
        } else {
            checkbox.build(tree)
        }
    }

    fn render_props(&self) -> RenderProps {
        RenderProps::default()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        &[]
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementTypeId::Div
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        None
    }
}

/// Create a checkbox with state from context
///
/// The checkbox uses reactive `State<bool>` for its checked status.
/// State changes automatically trigger visual updates via signals.
///
/// # Example
///
/// ```ignore
/// use blinc_cn::prelude::*;
///
/// fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
///     let checked = ctx.use_state_for("remember_me", false);
///
///     cn::checkbox(&checked)
///         .label("Remember me")
///         .on_change(|checked| println!("Checked: {}", checked))
/// }
/// ```
pub fn checkbox(state: &State<bool>) -> Checkbox {
    Checkbox::new(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_sizes() {
        assert_eq!(CheckboxSize::Small.size(), 14.0);
        assert_eq!(CheckboxSize::Medium.size(), 18.0);
        assert_eq!(CheckboxSize::Large.size(), 22.0);
    }

    #[test]
    fn test_checkbox_checkmark_sizes() {
        assert_eq!(CheckboxSize::Small.checkmark_size(), 10.0);
        assert_eq!(CheckboxSize::Medium.checkmark_size(), 12.0);
        assert_eq!(CheckboxSize::Large.checkmark_size(), 16.0);
    }
}
