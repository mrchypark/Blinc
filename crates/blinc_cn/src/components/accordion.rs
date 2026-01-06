//! Accordion component for expandable content sections
//!
//! A set of vertically stacked collapsible sections. Supports single-open
//! (only one section open at a time) or multi-open modes.
//!
//! Uses global animation scheduler - no context needed.
//!
//! # Example - Single Open
//!
//! ```ignore
//! use blinc_cn::prelude::*;
//!
//! fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
//!     cn::accordion()
//!         .item("section-1", "What is Blinc?", || {
//!             div().p(16.0).child(text("Blinc is a Rust UI framework..."))
//!         })
//!         .item("section-2", "How do I use it?", || {
//!             div().p(16.0).child(text("Start by creating a window..."))
//!         })
//!         .item("section-3", "Is it fast?", || {
//!             div().p(16.0).child(text("Yes, very fast!"))
//!         })
//! }
//! ```
//!
//! # Multi-Open Mode
//!
//! ```ignore
//! cn::accordion()
//!     .multi_open()  // Allow multiple sections open at once
//!     .item("a", "First Section", || content_a())
//!     .item("b", "Second Section", || content_b())
//! ```

use blinc_animation::{AnimatedValue, SpringConfig};
use blinc_core::context_state::BlincContextState;
use blinc_core::{use_state_keyed, SignalId, State};
use blinc_layout::div::ElementTypeId;
use blinc_layout::element::{CursorStyle, RenderProps};
use blinc_layout::layout_animation::LayoutAnimationConfig;
use blinc_layout::motion::{motion, SharedAnimatedValue};
use blinc_layout::prelude::*;
use blinc_layout::render_state::get_global_scheduler;
use blinc_layout::stateful::Stateful;
use blinc_layout::tree::{LayoutNodeId, LayoutTree};
use blinc_layout::InstanceKey;
use blinc_theme::{ColorToken, RadiusToken, ThemeState};
use std::cell::OnceCell;
use std::sync::{Arc, Mutex};

/// Chevron down SVG icon
const CHEVRON_DOWN_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>"#;

/// Chevron up SVG icon (for when section is open)
const CHEVRON_UP_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18 15-6-6-6 6"/></svg>"#;

/// Accordion mode - single or multi open
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionMode {
    /// Only one section can be open at a time (default)
    #[default]
    Single,
    /// Multiple sections can be open simultaneously
    Multi,
}

/// Accordion component - multiple collapsible sections
pub struct Accordion {
    /// The fully-built inner element
    inner: Stateful<()>,
}

impl ElementBuilder for Accordion {
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

    fn layout_style(&self) -> Option<&taffy::Style> {
        self.inner.layout_style()
    }
}

/// Content builder function type (cloneable via Arc)
type ContentBuilderFn = Arc<dyn Fn() -> Div + Send + Sync>;

/// Single accordion item - stores data, not built elements
#[derive(Clone)]
struct AccordionItem {
    key: String,
    label: String,
    content: ContentBuilderFn,
}

/// Runtime state for an accordion item (created during build)
#[derive(Clone)]
struct AccordionItemState {
    key: String,
    is_open: State<bool>,
    opacity_anim: SharedAnimatedValue,
}

/// Builder for creating Accordion components with fluent API
pub struct AccordionBuilder {
    instance_key: InstanceKey,
    mode: AccordionMode,
    spring_config: SpringConfig,
    initial_open: Option<String>,
    /// Item definitions (not yet built)
    items: Vec<AccordionItem>,
    /// Cached built accordion
    built: OnceCell<Accordion>,
}

impl AccordionBuilder {
    /// Create a new accordion builder
    ///
    /// Uses global animation scheduler - no context needed.
    pub fn new() -> Self {
        Self {
            instance_key: InstanceKey::new("accordion"),
            mode: AccordionMode::Single,
            spring_config: SpringConfig::snappy(),
            initial_open: None,
            items: Vec::new(),
            built: OnceCell::new(),
        }
    }

    /// Get or build the accordion
    fn get_or_build(&self) -> &Accordion {
        self.built.get_or_init(|| self.build_component())
        // self.built.get_or_init(|| {
        //     let theme = ThemeState::get();

        //     // Build container - we can't move items out, so build from scratch
        //     // This is a limitation - the builder should be consumed via build_component()
        //     let inner = div()
        //         .flex_col()
        //         .w_full()
        //         .rounded(theme.radius(RadiusToken::Md))
        //         .border(1.0, theme.color(ColorToken::Border));

        //     Accordion { inner }
        // })
    }

    /// Set to multi-open mode (multiple sections can be open at once)
    pub fn multi_open(mut self) -> Self {
        self.mode = AccordionMode::Multi;
        self
    }

    /// Set the initially open section (by key)
    pub fn default_open(mut self, key: impl Into<String>) -> Self {
        self.initial_open = Some(key.into());
        self
    }

    /// Set custom spring configuration for animations
    pub fn spring(mut self, config: SpringConfig) -> Self {
        self.spring_config = config;
        self
    }

    /// Add an accordion item
    ///
    /// # Arguments
    /// * `key` - Unique identifier for this section
    /// * `label` - Text shown in the trigger header
    /// * `content` - Function that builds the content when expanded
    pub fn item<F>(mut self, key: impl Into<String>, label: impl Into<String>, content: F) -> Self
    where
        F: Fn() -> Div + Send + Sync + 'static,
    {
        // Store item data - don't build yet
        self.items.push(AccordionItem {
            key: self.instance_key.derive(&key.into()),
            label: label.into(),
            content: Arc::new(content),
        });
        self
    }

    /// Build the final Accordion component
    pub fn build_component(&self) -> Accordion {
        let theme = ThemeState::get();

        // Get scheduler from global
        let scheduler = get_global_scheduler()
            .expect("Animation scheduler not initialized - call this after app starts");

        // Collect all signal IDs for the container's deps
        let mut all_signal_ids: Vec<SignalId> = Vec::new();

        // Build combined item data with runtime state - no mutex needed, just Vec
        let mut items_with_state: Vec<(AccordionItem, AccordionItemState)> = Vec::new();

        for item in &self.items {
            let is_initially_open = self.initial_open.as_ref() == Some(&item.key);

            // Create State<bool> using BlincContextState for reactivity
            let state_key = format!("{}_{}_open", self.instance_key.get(), item.key);
            let is_open: State<bool> =
                BlincContextState::get().use_state_keyed(&state_key, || is_initially_open);

            // Collect signal ID for container deps
            all_signal_ids.push(is_open.signal_id());

            // Get actual current state (may differ from initial if persisted)
            let actual_is_open = is_open.get();
            let actual_opacity = if actual_is_open { 1.0 } else { 0.0 };

            // Create opacity animation
            let opacity_anim: SharedAnimatedValue = Arc::new(Mutex::new(AnimatedValue::new(
                scheduler.clone(),
                actual_opacity,
                self.spring_config,
            )));

            let item_state = AccordionItemState {
                key: item.key.clone(),
                is_open,
                opacity_anim,
            };

            items_with_state.push((item.clone(), item_state));
        }

        // Theme colors
        let text_primary = theme.color(ColorToken::TextPrimary);
        let text_secondary = theme.color(ColorToken::TextSecondary);
        let border_color = theme.color(ColorToken::Border);
        let radius = theme.radius(RadiusToken::Md);

        let key_for_container = format!("{}_container", self.instance_key.get());
        let container_state_handle = use_shared_state_with(&key_for_container, ());

        let item_count = items_with_state.len();
        let mode = self.mode;

        // Clone all states for single-mode closing (needed in on_click)
        let all_item_states: Vec<AccordionItemState> =
            items_with_state.iter().map(|(_, s)| s.clone()).collect();

        // Build the entire accordion as a single Stateful that reacts to ALL item open states
        let accordion_stateful = Stateful::with_shared_state(container_state_handle)
            .deps(&all_signal_ids)
            .flex_col()
            .w_full()
            .on_state(move |_state: &(), container: &mut Div| {
                let mut content = div()
                    .flex_col()
                    .w_full()
                    .h_fit()
                    .rounded(radius)
                    .border(1.0, border_color);

                for (index, (item, item_state)) in items_with_state.iter().enumerate() {
                    let is_open = item_state.is_open.clone();
                    let opacity_anim = item_state.opacity_anim.clone();
                    let item_key = item_state.key.clone();

                    let content_fn = item.content.clone();
                    let label = item.label.clone();

                    // Clones for on_click closure
                    let is_open_for_click = is_open.clone();
                    let opacity_anim_for_click = opacity_anim.clone();
                    let all_states_for_click = all_item_states.clone();
                    let key_for_click = item_key.clone();

                    let section_is_open = is_open.get();

                    // Build content - conditionally shown with proper height
                    // // Note: motion() wrapper removed for now - layout debugging
                    // let collapsible_content = if section_is_open {

                    // } else {
                    //     // Empty placeholder when closed
                    //     div()
                    // };

                    // Build trigger - stateless div since container rebuilds on state change
                    let chevron_svg = if section_is_open {
                        CHEVRON_UP_SVG
                    } else {
                        CHEVRON_DOWN_SVG
                    };

                    let mut trigger = div()
                        .flex_row()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .cursor(CursorStyle::Pointer)
                        .child(
                            text(&label)
                                .size(14.0)
                                .weight(blinc_layout::div::FontWeight::Medium)
                                .color(text_primary)
                                .pointer_events_none(),
                        )
                        .child(svg(chevron_svg).size(16.0, 16.0).color(text_secondary))
                        .on_click(move |_| {
                            let current = is_open_for_click.get();
                            let new_state = !current;

                            // In single mode, close all other sections first
                            if mode == AccordionMode::Single && new_state {
                                for state in &all_states_for_click {
                                    if state.key != key_for_click && state.is_open.get() {
                                        state.is_open.set(false);
                                        state.opacity_anim.lock().unwrap().set_target(0.0);
                                    }
                                }
                            }

                            // Toggle this section
                            is_open_for_click.set(new_state);

                            let target_opacity = if new_state { 1.0 } else { 0.0 };
                            opacity_anim_for_click
                                .lock()
                                .unwrap()
                                .set_target(target_opacity);
                        });

                    if !section_is_open {
                        trigger = trigger.padding_y(Length::Px(12.0));
                    }

                    // Combine trigger and collapsible content
                    // The item_div has animate_layout for smooth height AND y position transitions
                    // Height animates open/close, Y position animates siblings moving
                    // Using stable key based on item_key so animation persists across rebuilds
                    let anim_key = format!("accordion-item-{}", item_key);
                    let mut item_div = div()
                        .padding_x(Length::Px(12.0))
                        .flex_col()
                        .flex_auto()
                        .overflow_clip()
                        .w_full()
                        .animate_layout(
                            LayoutAnimationConfig::all().with_key(anim_key).gentle(), // Use gentle spring for visible animation
                        )
                        .child(trigger);
                    //.child(collapsible_content);

                    if section_is_open {
                        item_div = item_div.padding_y(Length::Px(16.0)).child(content_fn());
                    }

                    content = content.child(item_div);

                    // Add separator between items (not after last)
                    if index < item_count - 1 {
                        content = content.child(hr_with_config(HrConfig {
                            color: border_color,
                            thickness: 1.0,
                            margin_y: 0.0,
                        }));
                    }
                }

                container.merge(content);
            });

        Accordion {
            inner: accordion_stateful,
        }
    }
}

impl Default for AccordionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementBuilder for AccordionBuilder {
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

    fn layout_style(&self) -> Option<&taffy::Style> {
        self.get_or_build().layout_style()
    }
}

/// Create an accordion
///
/// Uses global animation scheduler - no context needed.
///
/// # Example
///
/// ```ignore
/// cn::accordion()
///     .item("faq-1", "What is Blinc?", || {
///         div().p(16.0).child(text("A Rust UI framework"))
///     })
///     .item("faq-2", "Is it fast?", || {
///         div().p(16.0).child(text("Yes!"))
///     })
/// ```
pub fn accordion() -> AccordionBuilder {
    AccordionBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accordion_mode_default() {
        assert_eq!(AccordionMode::default(), AccordionMode::Single);
    }
}
