//! Card component for content containers
//!
//! A styled container with shadow and border for grouping related content.
//!
//! # Example
//!
//! ```ignore
//! use blinc_cn::prelude::*;
//!
//! // Simple card with content
//! cn::card()
//!     .child(text("Card content"))
//!
//! // Card with structured content using CardHeader and CardFooter
//! cn::card()
//!     .child(cn::card_header().title("Card Title").description("Description"))
//!     .child(text("Main content goes here"))
//!     .child(cn::card_footer().child(cn::button("Action")))
//!
//! // Card with custom styling (via Deref to Div)
//! cn::card()
//!     .shadow_lg()  // Larger shadow
//!     .p(32.0)      // Custom padding
//!     .child(text("Custom styled card"))
//! ```

use std::ops::{Deref, DerefMut};

use blinc_layout::div::{Div, ElementBuilder, ElementTypeId};
use blinc_layout::prelude::*;
use blinc_theme::{ColorToken, SpacingToken, ThemeState};

/// Card component for content containers
///
/// Implements `Deref` to `Div` for full customization.
pub struct Card {
    inner: Div,
}

impl Card {
    /// Create a new empty card
    pub fn new() -> Self {
        // All visual props from CSS: .cn-card { background, border, border-radius, padding, gap }
        let inner = div()
            .class("cn-card")
            .shadow_sm()
            .flex_col()
            .items_stretch();

        Self { inner }
    }

    /// Add content to the card body
    pub fn child(mut self, content: impl ElementBuilder + 'static) -> Self {
        self.inner = self.inner.child(content);
        self
    }

    // Forwarding methods for common Div operations

    /// Set width
    pub fn w(mut self, width: f32) -> Self {
        self.inner = self.inner.w(width);
        self
    }

    /// Set height
    pub fn h(mut self, height: f32) -> Self {
        self.inner = self.inner.h(height);
        self
    }

    /// Set full width
    pub fn w_full(mut self) -> Self {
        self.inner = self.inner.w_full();
        self
    }

    /// Set padding on all sides
    pub fn p(mut self, padding: f32) -> Self {
        self.inner = self.inner.p(padding);
        self
    }

    /// Set horizontal padding
    pub fn px(mut self, padding: f32) -> Self {
        self.inner = self.inner.px(padding);
        self
    }

    /// Set vertical padding
    pub fn py(mut self, padding: f32) -> Self {
        self.inner = self.inner.py(padding);
        self
    }

    /// Set margin on all sides
    pub fn m(mut self, margin: f32) -> Self {
        self.inner = self.inner.m(margin);
        self
    }

    /// Apply large shadow
    pub fn shadow_lg(mut self) -> Self {
        self.inner = self.inner.shadow_lg();
        self
    }

    /// Apply medium shadow
    pub fn shadow_md(mut self) -> Self {
        self.inner = self.inner.shadow_md();
        self
    }

    /// Set background color
    pub fn bg(mut self, color: blinc_core::Color) -> Self {
        self.inner = self.inner.bg(color);
        self
    }

    /// Add a CSS class for selector matching
    pub fn class(mut self, name: impl AsRef<str>) -> Self {
        self.inner = self.inner.class(name);
        self
    }

    /// Set the element ID for CSS selector matching
    pub fn id(mut self, id: &str) -> Self {
        self.inner = self.inner.id(id);
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Card {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Card {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for Card {
    fn build(&self, tree: &mut blinc_layout::tree::LayoutTree) -> blinc_layout::tree::LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> blinc_layout::element::RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn event_handlers(&self) -> Option<&blinc_layout::event_handler::EventHandlers> {
        ElementBuilder::event_handlers(&self.inner)
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        ElementBuilder::layout_style(&self.inner)
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementBuilder::element_type_id(&self.inner)
    }

    fn element_classes(&self) -> &[std::sync::Arc<str>] {
        self.inner.element_classes()
    }

    fn element_id(&self) -> Option<&str> {
        self.inner.element_id()
    }
}

/// Create an empty card
///
/// # Example
///
/// ```ignore
/// use blinc_cn::prelude::*;
///
/// cn::card()
///     .child(text("Content"))
/// ```
pub fn card() -> Card {
    Card::new()
}

// ============================================================================
// Card subcomponents for structured content
// ============================================================================

/// Card header section
pub struct CardHeader {
    inner: Div,
}

impl CardHeader {
    /// Create a new card header
    pub fn new() -> Self {
        let theme = ThemeState::get();
        let gap = theme.spacing_value(SpacingToken::Space1_5); // 6px
        let inner = div()
            .class("cn-card-header")
            .flex_col()
            .items_stretch()
            .w_full()
            .gap_px(gap);

        Self { inner }
    }

    /// Add a title
    pub fn title(mut self, title: impl ToString) -> Self {
        let theme = ThemeState::get();
        self.inner = self.inner.child(
            text(title)
                .size(18.0)
                .semibold()
                .color(theme.color(ColorToken::TextPrimary)),
        );
        self
    }

    /// Add a description
    pub fn description(mut self, desc: impl ToString) -> Self {
        let theme = ThemeState::get();
        self.inner = self.inner.child(
            text(desc)
                .size(14.0)
                .color(theme.color(ColorToken::TextSecondary)),
        );
        self
    }
}

impl Default for CardHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CardHeader {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CardHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for CardHeader {
    fn build(&self, tree: &mut blinc_layout::tree::LayoutTree) -> blinc_layout::tree::LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> blinc_layout::element::RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn event_handlers(&self) -> Option<&blinc_layout::event_handler::EventHandlers> {
        ElementBuilder::event_handlers(&self.inner)
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        ElementBuilder::layout_style(&self.inner)
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementBuilder::element_type_id(&self.inner)
    }

    fn element_classes(&self) -> &[std::sync::Arc<str>] {
        self.inner.element_classes()
    }

    fn element_id(&self) -> Option<&str> {
        self.inner.element_id()
    }
}

/// Create a card header
pub fn card_header() -> CardHeader {
    CardHeader::new()
}

/// Card content section - grows to fill available space
pub struct CardContent {
    inner: Div,
}

impl CardContent {
    /// Create a new card content section
    pub fn new() -> Self {
        let inner = div()
            .flex_col()
            .flex_1() // Grow to fill available space
            .w_full();

        Self { inner }
    }

    /// Add a child element
    pub fn child(mut self, content: impl ElementBuilder + 'static) -> Self {
        self.inner = self.inner.child(content);
        self
    }
}

impl Default for CardContent {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CardContent {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CardContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for CardContent {
    fn build(&self, tree: &mut blinc_layout::tree::LayoutTree) -> blinc_layout::tree::LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> blinc_layout::element::RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn event_handlers(&self) -> Option<&blinc_layout::event_handler::EventHandlers> {
        ElementBuilder::event_handlers(&self.inner)
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        ElementBuilder::layout_style(&self.inner)
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementBuilder::element_type_id(&self.inner)
    }

    fn element_classes(&self) -> &[std::sync::Arc<str>] {
        self.inner.element_classes()
    }

    fn element_id(&self) -> Option<&str> {
        self.inner.element_id()
    }
}

/// Create a card content section
pub fn card_content() -> CardContent {
    CardContent::new()
}

/// Card footer section
pub struct CardFooter {
    inner: Div,
}

impl CardFooter {
    /// Create a new card footer
    pub fn new() -> Self {
        let theme = ThemeState::get();
        let gap = theme.spacing_value(SpacingToken::Space2); // 8px
        let inner = div()
            .class("cn-card-footer")
            .flex_row()
            .w_full()
            .gap_px(gap)
            .justify_end();

        Self { inner }
    }

    /// Add a child element
    pub fn child(mut self, content: impl ElementBuilder + 'static) -> Self {
        self.inner = self.inner.child(content);
        self
    }
}

impl Default for CardFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CardFooter {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CardFooter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for CardFooter {
    fn build(&self, tree: &mut blinc_layout::tree::LayoutTree) -> blinc_layout::tree::LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> blinc_layout::element::RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn event_handlers(&self) -> Option<&blinc_layout::event_handler::EventHandlers> {
        ElementBuilder::event_handlers(&self.inner)
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        ElementBuilder::layout_style(&self.inner)
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementBuilder::element_type_id(&self.inner)
    }

    fn element_classes(&self) -> &[std::sync::Arc<str>] {
        self.inner.element_classes()
    }

    fn element_id(&self) -> Option<&str> {
        self.inner.element_id()
    }
}

/// Create a card footer
pub fn card_footer() -> CardFooter {
    CardFooter::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_theme() {
        let _ = ThemeState::try_get().unwrap_or_else(|| {
            ThemeState::init_default();
            ThemeState::get()
        });
    }

    #[test]
    fn test_card_default() {
        init_theme();
        let _ = card();
    }

    #[test]
    fn test_card_with_content() {
        init_theme();
        let _ = card().child(text("Content"));
    }

    #[test]
    fn test_card_defaults_to_stretch_children() {
        init_theme();
        let card = card();
        let style = card.layout_style().unwrap();

        assert_eq!(style.align_items, Some(taffy::AlignItems::Stretch));
    }

    #[test]
    fn test_card_header() {
        init_theme();
        let _ = card_header().title("Title").description("Description");
    }

    #[test]
    fn test_card_header_defaults_to_stretch_children() {
        init_theme();
        let header = card_header();
        let style = header.layout_style().unwrap();

        assert_eq!(style.align_items, Some(taffy::AlignItems::Stretch));
    }

    #[test]
    fn test_card_footer() {
        init_theme();
        let _ = card_footer();
    }

    #[test]
    fn test_card_wraps_direct_text_child_to_card_width() {
        init_theme();

        let long_text =
            "카드 안에서 긴 본문 텍스트는 카드 폭 안에서 자연스럽게 여러 줄로 줄바꿈되어야 합니다.";
        let ui = card().w(120.0).child(text(long_text));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(120.0, 400.0);

        let root = tree.root().unwrap();
        let children = tree.layout_tree.children(root);
        let text_bounds = tree
            .layout_tree
            .get_bounds(children[0], (0.0, 0.0))
            .unwrap();

        assert!(text_bounds.width <= 120.0);
        assert!(text_bounds.height > 20.0);
    }

    #[test]
    fn test_card_header_description_wraps_to_header_width() {
        init_theme();

        let long_text =
            "설명 텍스트도 헤더 내부 폭을 상속받아 단일 줄이 아니라 여러 줄로 측정되어야 합니다.";
        let ui = card()
            .w(120.0)
            .child(card_header().title("제목").description(long_text));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(120.0, 400.0);

        let root = tree.root().unwrap();
        let root_children = tree.layout_tree.children(root);
        let header = root_children[0];
        let header_children = tree.layout_tree.children(header);
        let description_bounds = tree
            .layout_tree
            .get_bounds(header_children[1], (0.0, 0.0))
            .unwrap();

        assert!(description_bounds.width <= 120.0);
        assert!(description_bounds.height > 20.0);
    }
}
