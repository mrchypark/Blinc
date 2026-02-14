//! Field component - themed wrapper for label + form control + helper/error text.

use std::cell::OnceCell;
use std::ops::{Deref, DerefMut};

use blinc_layout::div::ElementTypeId;
use blinc_layout::prelude::*;
use blinc_theme::{ColorToken, SpacingToken, ThemeState};

use super::label::{label, LabelSize};
use super::shared::SharedElement;

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

        let mut lbl = label(&config.label).size(LabelSize::Medium);
        if config.required {
            lbl = lbl.required();
        }
        if config.disabled {
            lbl = lbl.disabled(true);
        }
        container = container.child(lbl);

        for child in config.children {
            container = container.child(child);
        }

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
    pub(crate) required: bool,
    pub(crate) disabled: bool,
    pub(crate) description: Option<String>,
    pub(crate) error: Option<String>,
    gap: Option<f32>,
    children: Vec<SharedElement>,
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

    pub fn child(mut self, child: impl ElementBuilder + 'static) -> Self {
        self.config.children.push(SharedElement::new(child));
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
}
