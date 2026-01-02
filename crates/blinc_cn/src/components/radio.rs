//! Radio Group component for single-selection from multiple options
//!
//! A themed radio button group with smooth selection transitions.
//! Uses State<String> from context for reactive state management.
//!
//! # Example
//!
//! ```ignore
//! use blinc_cn::prelude::*;
//!
//! fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
//!     // Create radio state from context
//!     let selected = ctx.use_state_for("color", "red".to_string());
//!
//!     cn::radio_group(&selected)
//!         .option("red", "Red")
//!         .option("green", "Green")
//!         .option("blue", "Blue")
//!         .on_change(|value| println!("Selected: {}", value))
//! }
//!
//! // Horizontal layout
//! cn::radio_group(&selected)
//!     .horizontal()
//!     .option("yes", "Yes")
//!     .option("no", "No")
//!
//! // With label
//! cn::radio_group(&size)
//!     .label("Select size")
//!     .option("sm", "Small")
//!     .option("md", "Medium")
//!     .option("lg", "Large")
//!
//! // Disabled
//! cn::radio_group(&selected)
//!     .disabled(true)
//!     .option("a", "Option A")
//!     .option("b", "Option B")
//! ```

use blinc_core::{Color, State, Transform};
use blinc_layout::div::ElementTypeId;
use blinc_layout::element::RenderProps;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_theme::{ColorToken, SpacingToken, ThemeState};
use std::sync::Arc;

use super::label::{label, LabelSize};

/// Radio button size variants
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioSize {
    /// Small radio (14px)
    Small,
    /// Medium radio (18px)
    #[default]
    Medium,
    /// Large radio (22px)
    Large,
}

impl RadioSize {
    fn outer_size(&self) -> f32 {
        match self {
            RadioSize::Small => 14.0,
            RadioSize::Medium => 18.0,
            RadioSize::Large => 22.0,
        }
    }

    fn inner_size(&self) -> f32 {
        match self {
            RadioSize::Small => 6.0,
            RadioSize::Medium => 8.0,
            RadioSize::Large => 10.0,
        }
    }

    fn border_width(&self) -> f32 {
        match self {
            RadioSize::Small => 1.5,
            RadioSize::Medium => 2.0,
            RadioSize::Large => 2.0,
        }
    }
}

/// Layout direction for radio options
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioLayout {
    /// Options stacked vertically (default)
    #[default]
    Vertical,
    /// Options arranged horizontally
    Horizontal,
}

/// A radio option with value and label
#[derive(Clone)]
struct RadioOption {
    value: String,
    label: String,
    disabled: bool,
}

/// Radio Group component
///
/// A group of radio buttons where only one can be selected at a time.
/// Uses State<String> from context for reactive state management.
pub struct RadioGroup {
    selected: State<String>,
    options: Vec<RadioOption>,
    size: RadioSize,
    layout: RadioLayout,
    label: Option<String>,
    disabled: bool,
    gap: Option<f32>,
    // Colors
    selected_color: Option<Color>,
    border_color: Option<Color>,
    hover_border_color: Option<Color>,
    // Callback
    on_change: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl RadioGroup {
    /// Create a new radio group with state from context
    ///
    /// # Example
    /// ```ignore
    /// let selected = ctx.use_state_for("choice", "option1".to_string());
    /// cn::radio_group(&selected)
    ///     .option("option1", "First Option")
    ///     .option("option2", "Second Option")
    /// ```
    pub fn new(selected: &State<String>) -> Self {
        Self {
            selected: selected.clone(),
            options: Vec::new(),
            size: RadioSize::default(),
            layout: RadioLayout::default(),
            label: None,
            disabled: false,
            gap: None,
            selected_color: None,
            border_color: None,
            hover_border_color: None,
            on_change: None,
        }
    }

    /// Add an option to the radio group
    pub fn option(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(RadioOption {
            value: value.into(),
            label: label.into(),
            disabled: false,
        });
        self
    }

    /// Add a disabled option to the radio group
    pub fn option_disabled(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(RadioOption {
            value: value.into(),
            label: label.into(),
            disabled: true,
        });
        self
    }

    /// Set the radio button size
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }

    /// Set the layout direction
    pub fn layout(mut self, layout: RadioLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Use horizontal layout
    pub fn horizontal(mut self) -> Self {
        self.layout = RadioLayout::Horizontal;
        self
    }

    /// Use vertical layout (default)
    pub fn vertical(mut self) -> Self {
        self.layout = RadioLayout::Vertical;
        self
    }

    /// Add a label above the radio group
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Disable all options
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the gap between options
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Set the selected indicator color
    pub fn selected_color(mut self, color: impl Into<Color>) -> Self {
        self.selected_color = Some(color.into());
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

    /// Set the change callback
    ///
    /// Called when a different option is selected, with the new value.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }

    /// Build a single radio button
    fn build_radio_button(
        &self,
        option: &RadioOption,
        theme: &ThemeState,
    ) -> Stateful<ButtonState> {
        let outer_size = self.size.outer_size();
        let inner_size = self.size.inner_size();
        let border_width = self.size.border_width();

        // Get colors
        let selected_color = self
            .selected_color
            .unwrap_or_else(|| theme.color(ColorToken::Primary));
        let border = self
            .border_color
            .unwrap_or_else(|| theme.color(ColorToken::Border));
        let hover_border = self
            .hover_border_color
            .unwrap_or_else(|| theme.color(ColorToken::BorderHover));

        let option_value = option.value.clone();
        let option_disabled = option.disabled || self.disabled;
        let option_label = option.label.clone();
        let selected_state = self.selected.clone();
        let selected_state_for_click = self.selected.clone();
        let on_change = self.on_change.clone();
        let value_for_click = option.value.clone();

        let label_color = if option_disabled {
            theme.color(ColorToken::TextTertiary)
        } else {
            theme.color(ColorToken::TextPrimary)
        };

        // Build the radio button circle
        let mut radio = Stateful::new(ButtonState::Idle)
            .flex_row()
            .gap(8.0)
            .items_center()
            .cursor_pointer()
            .deps(&[selected_state.signal_id()]);

        if option_disabled {
            radio = radio.opacity(0.5);
        }

        // State callback for visual changes
        radio = radio.on_state(move |state: &ButtonState, container: &mut Div| {
            let is_selected = selected_state.get() == option_value;
            let is_hovered = matches!(state, ButtonState::Hovered | ButtonState::Pressed);
            let is_pressed = matches!(state, ButtonState::Pressed);

            // Border color based on state
            let current_border = if is_selected {
                selected_color
            } else if is_hovered && !option_disabled {
                hover_border
            } else {
                border
            };

            // Scale effect on hover
            let scale = if is_hovered && !option_disabled {
                1.05
            } else {
                1.0
            };

            // Scale effect on press
            let inner_scale = if is_pressed && !option_disabled {
                0.8
            } else {
                1.0
            };

            // Build the radio circle (outer ring)
            let mut circle = div()
                .w(outer_size)
                .h(outer_size)
                .rounded(outer_size / 2.0)
                .border(border_width, current_border)
                .items_center()
                .justify_center()
                .transform(Transform::scale(scale, scale));

            // Add inner dot if selected
            if is_selected {
                let inner_dot = div()
                    .w(inner_size)
                    .h(inner_size)
                    .rounded(inner_size / 2.0)
                    .bg(selected_color)
                    .transform(Transform::scale(inner_scale, inner_scale));
                circle = circle.child(inner_dot);
            }

            // Build with label
            let visual = div()
                .flex_row()
                .gap(8.0)
                .items_center()
                .child(circle)
                .child(text(&option_label).size(14.0).color(label_color));

            container.merge(visual);
        });

        // Click handler
        radio = radio.on_click(move |_| {
            if option_disabled {
                return;
            }

            let current = selected_state_for_click.get();
            if current != value_for_click {
                selected_state_for_click.set(value_for_click.clone());
                if let Some(ref callback) = on_change {
                    callback(&value_for_click);
                }
            }
        });

        radio
    }
}

impl ElementBuilder for RadioGroup {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        let theme = ThemeState::get();
        let default_gap = theme.spacing_value(SpacingToken::Space3);
        let gap = self.gap.unwrap_or(default_gap);

        // Build options container
        let mut options_container = match self.layout {
            RadioLayout::Vertical => div().flex_col().gap(gap),
            RadioLayout::Horizontal => div().flex_row().gap(gap).flex_wrap(),
        };

        // Add each radio button
        for option in &self.options {
            options_container = options_container.child(self.build_radio_button(option, &theme));
        }

        // If there's a label, wrap everything
        if let Some(ref label_text) = self.label {
            let spacing = theme.spacing_value(SpacingToken::Space2);
            let mut lbl = label(label_text).size(LabelSize::Medium);
            if self.disabled {
                lbl = lbl.disabled(true);
            }

            div()
                .flex_col()
                .gap(spacing)
                .child(lbl)
                .child(options_container)
                .build(tree)
        } else {
            options_container.build(tree)
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
}

/// Create a radio group with state from context
///
/// # Example
///
/// ```ignore
/// use blinc_cn::prelude::*;
///
/// let choice = ctx.use_state_for("choice", "a".to_string());
/// cn::radio_group(&choice)
///     .label("Pick one")
///     .option("a", "Option A")
///     .option("b", "Option B")
///     .option("c", "Option C")
/// ```
pub fn radio_group(selected: &State<String>) -> RadioGroup {
    RadioGroup::new(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radio_sizes() {
        assert_eq!(RadioSize::Small.outer_size(), 14.0);
        assert_eq!(RadioSize::Medium.outer_size(), 18.0);
        assert_eq!(RadioSize::Large.outer_size(), 22.0);
    }

    #[test]
    fn test_radio_inner_sizes() {
        assert_eq!(RadioSize::Small.inner_size(), 6.0);
        assert_eq!(RadioSize::Medium.inner_size(), 8.0);
        assert_eq!(RadioSize::Large.inner_size(), 10.0);
    }

    #[test]
    fn test_radio_border_widths() {
        assert_eq!(RadioSize::Small.border_width(), 1.5);
        assert_eq!(RadioSize::Medium.border_width(), 2.0);
        assert_eq!(RadioSize::Large.border_width(), 2.0);
    }

    fn init_theme() {
        let _ = ThemeState::try_get().unwrap_or_else(|| {
            ThemeState::init_default();
            ThemeState::get()
        });
    }

    // Note: RadioGroup builder test requires State which needs context
    // The RadioGroup API is tested through the size/layout tests above
}
