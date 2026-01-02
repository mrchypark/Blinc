//! Select component for dropdown value selection
//!
//! A themed select dropdown with click-to-open and keyboard navigation.
//! Uses state-driven reactivity for proper persistence across UI rebuilds.
//!
//! # Example
//!
//! ```ignore
//! use blinc_cn::prelude::*;
//!
//! fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
//!     let fruit = ctx.use_state_keyed("fruit", || "apple".to_string());
//!
//!     cn::select(&fruit)
//!         .placeholder("Choose a fruit...")
//!         .option("apple", "Apple")
//!         .option("banana", "Banana")
//!         .option("cherry", "Cherry")
//!         .on_change(|value| println!("Selected: {}", value))
//! }
//!
//! // Different sizes
//! cn::select(&value)
//!     .size(SelectSize::Large)
//!
//! // Disabled state
//! cn::select(&value)
//!     .disabled(true)
//!
//! // With label
//! cn::select(&value)
//!     .label("Favorite Fruit")
//! ```

use std::cell::OnceCell;
use std::sync::Arc;

use blinc_core::context_state::BlincContextState;
use blinc_core::State;
use blinc_layout::div::ElementTypeId;
use blinc_layout::element::{CursorStyle, RenderProps};
use blinc_layout::overlay_state::get_overlay_manager;
use blinc_layout::prelude::*;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_layout::widgets::overlay::{OverlayHandle, OverlayManagerExt};
use blinc_theme::{ColorToken, RadiusToken, SpacingToken, ThemeState};

use super::label::{label, LabelSize};

/// Select size variants
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectSize {
    /// Small select (height: 32px, text: 13px)
    Small,
    /// Medium select (height: 40px, text: 14px)
    #[default]
    Medium,
    /// Large select (height: 48px, text: 16px)
    Large,
}

impl SelectSize {
    /// Get the height for this size
    fn height(&self) -> f32 {
        match self {
            SelectSize::Small => 32.0,
            SelectSize::Medium => 40.0,
            SelectSize::Large => 48.0,
        }
    }

    /// Get the font size for this size
    fn font_size(&self) -> f32 {
        match self {
            SelectSize::Small => 13.0,
            SelectSize::Medium => 14.0,
            SelectSize::Large => 16.0,
        }
    }

    /// Get the padding for this size
    fn padding(&self) -> f32 {
        match self {
            SelectSize::Small => 8.0,
            SelectSize::Medium => 12.0,
            SelectSize::Large => 16.0,
        }
    }
}

/// An option in the select dropdown
#[derive(Clone, Debug)]
pub struct SelectOption {
    /// The value (stored in state when selected)
    pub value: String,
    /// The display label shown in UI
    pub label: String,
    /// Whether this option is disabled
    pub disabled: bool,
}

impl SelectOption {
    /// Create a new option with value and label
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Mark this option as disabled
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Select component
///
/// A dropdown select with click-to-open and item selection.
/// Uses state-driven reactivity for proper persistence across UI rebuilds.
pub struct Select {
    /// The fully-built inner element
    inner: Div,
}

impl Select {
    /// Create from a full configuration
    fn from_config(config: SelectConfig) -> Self {
        let theme = ThemeState::get();
        let height = config.size.height();
        let font_size = config.size.font_size();
        let padding = config.size.padding();
        let radius = theme.radius(RadiusToken::Md);

        // Colors
        let bg = theme.color(ColorToken::Surface);
        let border = theme.color(ColorToken::Border);
        let border_hover = theme.color(ColorToken::BorderHover);
        let text_color = theme.color(ColorToken::TextPrimary);
        let text_tertiary = theme.color(ColorToken::TextTertiary);
        let surface_elevated = theme.color(ColorToken::SurfaceElevated);

        let disabled = config.disabled;

        // Create internal open_state using the singleton (tracks whether dropdown should be shown)
        let open_key = format!("_select_open_{}", config.instance_key);
        let open_state = BlincContextState::get().use_state_keyed(&open_key, || false);

        // Store overlay handle to track the dropdown overlay
        let handle_key = format!("_select_handle_{}", config.instance_key);
        let overlay_handle_state: State<Option<u64>> =
            BlincContextState::get().use_state_keyed(&handle_key, || None);

        // Store dropdown width for overlay
        let dropdown_width = config.width.unwrap_or(200.0);

        // Clones for closures
        let value_state_for_display = config.value_state.clone();
        let open_state_for_display = open_state.clone();
        let options_for_display = config.options.clone();
        let placeholder_for_display = config.placeholder.clone();

        // Chevron SVG (down arrow)
        let chevron_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>"#;

        // Build dropdown options
        let options = config.options.clone();
        let on_change = config.on_change.clone();
        let value_state_for_options = config.value_state.clone();
        let open_state_for_click = open_state.clone();
        let overlay_handle_for_click = overlay_handle_state.clone();

        // Use Stateful with () as we only need signal deps, not FSM state transitions
        // The click handler is on the Stateful itself (not the inner div) so it gets registered
        // Use w_full() to ensure the Stateful takes the same width as its parent container
        let select_element = Stateful::<()>::new(())
            .deps(&[config.value_state.signal_id(), open_state.signal_id()])
            .w_full()
            .h(height)
            .cursor_pointer()
            .on_state(move |_state: &(), container: &mut Div| {
                let is_open = open_state_for_display.get();

                // Get current display value
                let current_val = value_state_for_display.get();
                let current_lbl = options_for_display
                    .iter()
                    .find(|opt| opt.value == current_val)
                    .map(|opt| opt.label.clone());

                let disp_text = current_lbl.clone().unwrap_or_else(|| {
                    placeholder_for_display
                        .clone()
                        .unwrap_or_else(|| "Select...".to_string())
                });
                let is_placeholder = current_lbl.is_none();
                let text_clr = if is_placeholder {
                    text_tertiary
                } else {
                    text_color
                };
                let bdr = if is_open { border_hover } else { border };

                // Build trigger (visual only - click handler is on Stateful)
                let trigger = div()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(height)
                    .p_px(padding)
                    .bg(bg)
                    .border(1.0, bdr)
                    .rounded(radius)
                    .child(text(&disp_text).size(font_size).color(text_clr))
                    .child(
                        svg(chevron_svg)
                            .size(16.0, 16.0)
                            .tint(text_tertiary)
                            .ml(1.0),
                    );

                let main_container = div().relative().w_full().child(trigger);

                container.merge(main_container);
            })
            .on_click(move |ctx| {
                let is_currently_open = open_state_for_click.get();

                if is_currently_open {
                    // Close the dropdown
                    if let Some(handle_id) = overlay_handle_for_click.get() {
                        let mgr = get_overlay_manager();
                        mgr.close(OverlayHandle::from_raw(handle_id));
                    }
                    open_state_for_click.set(false);
                    overlay_handle_for_click.set(None);
                } else {
                    // Calculate dropdown position from element bounds
                    // Use mouse position minus local offset to get element's screen position
                    // (bounds_x/y don't account for scroll offset correctly)
                    // let trigger_screen_x = ctx.mouse_x - ctx.local_x;
                    let trigger_screen_y = ctx.mouse_y - ctx.local_y;
                    // Position dropdown below the trigger, centered horizontally
                    // let trigger_center_x = trigger_screen_x + ctx.bounds_width / 2.0;
                    let dropdown_x = ctx.bounds_x - ctx.bounds_width / 2.0;
                    let dropdown_y = ctx.bounds_y + ctx.bounds_height - 4.0;

                    // Clone values for the dropdown content closure
                    let opts = options.clone();
                    let val_state = value_state_for_options.clone();
                    let open_st = open_state_for_click.clone();
                    let handle_st = overlay_handle_for_click.clone();
                    let on_chg = on_change.clone();
                    let current_selected = val_state.get();
                    let dw = dropdown_width;

                    // Estimate dropdown height for hit testing
                    // Each option: py(8.0) * 2 + font_size + padding
                    let option_height = 16.0 + font_size + padding * 2.0;
                    let estimated_height =
                        (option_height * opts.len() as f32).min(200.0) + 2.0; // +2 for border

                    // Clone for on_close callback
                    let open_state_for_close = open_state_for_click.clone();
                    let handle_state_for_close = overlay_handle_for_click.clone();

                    // Show dropdown via overlay manager
                    let mgr = get_overlay_manager();
                    let handle = mgr
                        .dropdown()
                        .at(dropdown_x, dropdown_y)
                        .size(dropdown_width, estimated_height)
                        .dismiss_on_escape(true)
                        .content(move || {
                            build_dropdown_content(
                                &opts,
                                &current_selected,
                                &val_state,
                                &open_st,
                                &handle_st,
                                &on_chg,
                                dw,
                                font_size,
                                padding,
                                radius,
                                bg,
                                border,
                                text_color,
                                text_tertiary,
                                surface_elevated,
                            )
                        })
                        .on_close(move || {
                            // Sync state when dropdown is dismissed externally
                            open_state_for_close.set(false);
                            handle_state_for_close.set(None);
                        })
                        .show();

                    open_state_for_click.set(true);
                    overlay_handle_for_click.set(Some(handle.id()));
                }
            });

        // Build the outer container with optional label
        let mut select_container = div().w_full().child(select_element);

        // Apply width if specified
        if let Some(w) = config.width {
            select_container = select_container.w(w);
        }

        if disabled {
            select_container = select_container.opacity(0.5);
        }

        // If there's a label, wrap in a container
        let inner = if let Some(ref label_text) = config.label {
            let spacing = theme.spacing_value(SpacingToken::Space2);
            let mut outer = div().flex_col().gap_px(spacing);

            if let Some(w) = config.width {
                outer = outer.w(w);
            } else {
                outer = outer.w_fit();
            }

            let mut lbl = label(label_text).size(LabelSize::Medium);
            if disabled {
                lbl = lbl.disabled(true);
            }

            outer = outer.child(lbl).child(select_container);
            outer
        } else {
            select_container
        };

        Self { inner }
    }
}

impl ElementBuilder for Select {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn element_type_id(&self) -> ElementTypeId {
        self.inner.element_type_id()
    }
}

/// Internal configuration for building a Select
#[derive(Clone)]
struct SelectConfig {
    value_state: State<String>,
    /// Unique instance key for this select (generated at creation time)
    instance_key: String,
    options: Vec<SelectOption>,
    placeholder: Option<String>,
    label: Option<String>,
    size: SelectSize,
    disabled: bool,
    width: Option<f32>,
    on_change: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl SelectConfig {
    fn new(value_state: State<String>, instance_key: String) -> Self {
        Self {
            value_state,
            instance_key,
            options: Vec::new(),
            placeholder: None,
            label: None,
            size: SelectSize::default(),
            disabled: false,
            width: None,
            on_change: None,
        }
    }
}

/// Builder for creating Select components with fluent API
pub struct SelectBuilder {
    config: SelectConfig,
    /// Cached built Select - built lazily on first access
    built: OnceCell<Select>,
}

impl SelectBuilder {
    /// Create a new select builder with value state
    ///
    /// The open state is managed internally using the global context singleton.
    /// Uses `#[track_caller]` to generate a unique instance key based on the call site.
    #[track_caller]
    pub fn new(value_state: &State<String>) -> Self {
        let loc = std::panic::Location::caller();
        let instance_key = format!(
            "{}:{}:{}:{}",
            loc.file(),
            loc.line(),
            loc.column(),
            value_state.signal_id().to_raw()
        );
        Self {
            config: SelectConfig::new(value_state.clone(), instance_key),
            built: OnceCell::new(),
        }
    }

    /// Get or build the inner Select
    fn get_or_build(&self) -> &Select {
        self.built
            .get_or_init(|| Select::from_config(self.config.clone()))
    }

    /// Add an option with value and label
    pub fn option(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.config.options.push(SelectOption::new(value, label));
        self
    }

    /// Add a disabled option
    pub fn option_disabled(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.config
            .options
            .push(SelectOption::new(value, label).disabled());
        self
    }

    /// Add multiple options
    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption>) -> Self {
        self.config.options.extend(options);
        self
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.config.placeholder = Some(placeholder.into());
        self
    }

    /// Add a label above the select
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.config.label = Some(label.into());
        self
    }

    /// Set the select size
    pub fn size(mut self, size: SelectSize) -> Self {
        self.config.size = size;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// Set a fixed width
    pub fn w(mut self, width: f32) -> Self {
        self.config.width = Some(width);
        self
    }

    /// Set the change callback
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.config.on_change = Some(Arc::new(callback));
        self
    }
}

impl ElementBuilder for SelectBuilder {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        self.get_or_build().build(tree)
    }

    fn render_props(&self) -> RenderProps {
        self.get_or_build().render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.get_or_build().children_builders()
    }

    fn element_type_id(&self) -> ElementTypeId {
        self.get_or_build().element_type_id()
    }
}

/// Create a select with value state
///
/// The select uses state-driven reactivity - changes to the value state
/// will trigger a rebuild of the component. The open/closed state is
/// managed internally using the global context singleton.
///
/// # Example
///
/// ```ignore
/// use blinc_cn::prelude::*;
///
/// fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
///     let fruit = ctx.use_state_keyed("fruit", || "apple".to_string());
///
///     cn::select(&fruit)
///         .placeholder("Choose a fruit...")
///         .option("apple", "Apple")
///         .option("banana", "Banana")
///         .on_change(|v| println!("Selected: {}", v))
/// }
/// ```
#[track_caller]
pub fn select(value_state: &State<String>) -> SelectBuilder {
    SelectBuilder::new(value_state)
}

/// Build the dropdown content for the overlay
///
/// This is extracted as a separate function to be called from the overlay content closure.
#[allow(clippy::too_many_arguments)]
fn build_dropdown_content(
    options: &[SelectOption],
    current_selected: &str,
    value_state: &State<String>,
    open_state: &State<bool>,
    overlay_handle_state: &State<Option<u64>>,
    on_change: &Option<Arc<dyn Fn(&str) + Send + Sync>>,
    width: f32,
    font_size: f32,
    padding: f32,
    radius: f32,
    bg: blinc_core::Color,
    border: blinc_core::Color,
    text_color: blinc_core::Color,
    text_tertiary: blinc_core::Color,
    surface_elevated: blinc_core::Color,
) -> Div {
    let mut dropdown_div = div()
        .flex_col()
        .w(width)
        .bg(bg)
        .border(1.0, border)
        .rounded(radius)
        .shadow_md()
        .overflow_clip()
        .max_h(200.0);

    for opt in options {
        let opt_value = opt.value.clone();
        let opt_label = opt.label.clone();
        let is_selected = opt_value == current_selected;
        let is_opt_disabled = opt.disabled;

        let value_state_for_opt = value_state.clone();
        let open_state_for_opt = open_state.clone();
        let handle_state_for_opt = overlay_handle_state.clone();
        let on_change_for_opt = on_change.clone();
        let opt_value_for_click = opt_value.clone();

        // Background color - selected items are highlighted
        let opt_bg = if is_selected { surface_elevated } else { bg };

        let option_text_color = if is_opt_disabled {
            text_tertiary
        } else {
            text_color
        };

        // Build option item
        let option_item = div()
            .w_full()
            .flex_row()
            .items_center()
            .h_fit()
            .py(8.0)
            .p_px(padding)
            .bg(opt_bg)
            .cursor(if is_opt_disabled {
                CursorStyle::NotAllowed
            } else {
                CursorStyle::Pointer
            })
            .child(text(&opt_label).size(font_size).color(option_text_color))
            .on_click(move |_ctx| {
                if !is_opt_disabled {
                    // Set the new value
                    value_state_for_opt.set(opt_value_for_click.clone());

                    // Close the overlay
                    if let Some(handle_id) = handle_state_for_opt.get() {
                        let mgr = get_overlay_manager();
                        mgr.close(OverlayHandle::from_raw(handle_id));
                    }
                    open_state_for_opt.set(false);
                    handle_state_for_opt.set(None);

                    // Call on_change callback
                    if let Some(ref cb) = on_change_for_opt {
                        cb(&opt_value_for_click);
                    }
                }
            });

        dropdown_div = dropdown_div.child(option_item);
    }

    dropdown_div
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_sizes() {
        assert_eq!(SelectSize::Small.height(), 32.0);
        assert_eq!(SelectSize::Medium.height(), 40.0);
        assert_eq!(SelectSize::Large.height(), 48.0);
    }

    #[test]
    fn test_select_font_sizes() {
        assert_eq!(SelectSize::Small.font_size(), 13.0);
        assert_eq!(SelectSize::Medium.font_size(), 14.0);
        assert_eq!(SelectSize::Large.font_size(), 16.0);
    }

    #[test]
    fn test_select_option() {
        let opt = SelectOption::new("value", "Label");
        assert_eq!(opt.value, "value");
        assert_eq!(opt.label, "Label");
        assert!(!opt.disabled);

        let disabled_opt = opt.disabled();
        assert!(disabled_opt.disabled);
    }
}
