//! Field component - themed wrapper for label + form control + helper/error text.

use std::cell::OnceCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use blinc_layout::div::ElementTypeId;
use blinc_layout::prelude::*;
use blinc_theme::{ColorToken, SpacingToken, ThemeState};

use super::label::{label, LabelSize};

/// Styled field component with label + control + helper text.
pub struct Field {
    inner: Div,
}

impl Field {
    fn with_config(config: FieldConfig) -> Self {
        let theme = ThemeState::get();
        let typography = theme.typography();
        let gap = config
            .gap
            .unwrap_or_else(|| theme.spacing_value(SpacingToken::Space2));

        let mut container = div().flex_col().h_fit().gap_px(gap);
        if config.full_width {
            container = container.w_full();
        } else if let Some(width) = config.width {
            container = container.w(width);
        }
        if let Some(max_width) = config.max_width {
            container = container.max_w(max_width);
        }

        let mut lbl = label(&config.label).size(config.label_size);
        if config.required {
            lbl = lbl.required();
        }
        if config.disabled {
            container = container.opacity(0.6).pointer_events_none();
        }
        container = container.child(lbl);

        container = container.children(config.children);

        if let Some(error_text) = config.error {
            container = container.child(
                text(error_text)
                    .size(typography.text_xs)
                    .color(theme.color(ColorToken::Error)),
            );
        } else if let Some(description_text) = config.description {
            container = container.child(
                text(description_text)
                    .size(typography.text_xs)
                    .color(theme.color(ColorToken::TextTertiary)),
            );
        }

        Self { inner: container }
    }
}

impl Deref for Field {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Field {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for Field {
    fn build(&self, tree: &mut blinc_layout::tree::LayoutTree) -> blinc_layout::tree::LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> blinc_layout::element::RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn element_type_id(&self) -> ElementTypeId {
        self.inner.element_type_id()
    }
}

#[derive(Clone, Default)]
pub(crate) struct FieldConfig {
    label: String,
    pub(crate) label_size: LabelSize,
    pub(crate) required: bool,
    pub(crate) disabled: bool,
    pub(crate) description: Option<String>,
    pub(crate) error: Option<String>,
    width: Option<f32>,
    max_width: Option<f32>,
    full_width: bool,
    gap: Option<f32>,
    children: Vec<Arc<dyn ElementBuilder>>,
}

/// Builder for `Field`.
pub struct FieldBuilder {
    pub(crate) config: FieldConfig,
    built: OnceCell<Field>,
}

impl FieldBuilder {
    pub fn new(label_text: impl Into<String>) -> Self {
        Self {
            config: FieldConfig {
                label: label_text.into(),
                ..Default::default()
            },
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &Field {
        self.built
            .get_or_init(|| Field::with_config(self.config.clone()))
    }

    pub fn required(mut self) -> Self {
        self.config.required = true;
        self
    }

    pub fn label_size(mut self, size: LabelSize) -> Self {
        self.config.label_size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.config.description = Some(description.into());
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.config.error = Some(error.into());
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.config.gap = Some(gap);
        self
    }

    pub fn w(mut self, width: f32) -> Self {
        self.config.width = Some(width);
        self.config.full_width = false;
        self
    }

    pub fn max_w(mut self, max_width: f32) -> Self {
        self.config.max_width = Some(max_width);
        self
    }

    pub fn w_full(mut self) -> Self {
        self.config.full_width = true;
        self.config.width = None;
        self
    }

    pub fn when(self, condition: bool, transform: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            transform(self)
        } else {
            self
        }
    }

    pub fn child(mut self, child: impl ElementBuilder + 'static) -> Self {
        self.config.children.push(Arc::new(child));
        self
    }

    pub fn build_component(self) -> Field {
        Field::with_config(self.config)
    }
}

impl ElementBuilder for FieldBuilder {
    fn build(&self, tree: &mut blinc_layout::tree::LayoutTree) -> blinc_layout::tree::LayoutNodeId {
        self.get_or_build().build(tree)
    }

    fn render_props(&self) -> blinc_layout::element::RenderProps {
        self.get_or_build().render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.get_or_build().children_builders()
    }

    fn element_type_id(&self) -> ElementTypeId {
        self.get_or_build().element_type_id()
    }
}

pub fn field(label_text: impl Into<String>) -> FieldBuilder {
    FieldBuilder::new(label_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_builder_sets_required_and_error() {
        let field = field("Email").required().error("Required");
        assert!(field.config.required);
        assert_eq!(field.config.error.as_deref(), Some("Required"));
    }

    #[test]
    fn test_field_builder_sets_description() {
        let field = field("Email").description("We'll never share your email.");
        assert_eq!(
            field.config.description.as_deref(),
            Some("We'll never share your email.")
        );
    }

    #[test]
    fn test_field_builder_sets_label_size() {
        let field = field("Email").label_size(LabelSize::Small);
        assert_eq!(field.config.label_size, LabelSize::Small);
    }

    #[test]
    fn test_field_builder_sets_width_controls() {
        let field = field("Email").w(320.0).max_w(480.0).w_full();
        assert_eq!(field.config.width, None);
        assert_eq!(field.config.max_width, Some(480.0));
        assert!(field.config.full_width);
    }
}
