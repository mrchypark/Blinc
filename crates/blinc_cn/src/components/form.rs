//! Form component - themed vertical layout wrapper for field composition.

use std::cell::OnceCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use blinc_layout::div::ElementTypeId;
use blinc_layout::prelude::*;
use blinc_theme::{SpacingToken, ThemeState};

#[derive(Clone)]
struct SharedElement(Arc<dyn ElementBuilder>);

impl ElementBuilder for SharedElement {
    fn build(&self, tree: &mut blinc_layout::tree::LayoutTree) -> blinc_layout::tree::LayoutNodeId {
        self.0.build(tree)
    }

    fn render_props(&self) -> blinc_layout::element::RenderProps {
        self.0.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.0.children_builders()
    }

    fn element_type_id(&self) -> ElementTypeId {
        self.0.element_type_id()
    }
}

/// Styled form container.
pub struct Form {
    inner: Div,
}

impl Form {
    fn with_config(config: FormConfig) -> Self {
        let theme = ThemeState::get();
        let spacing = config
            .spacing
            .unwrap_or_else(|| theme.spacing_value(SpacingToken::Space4));

        let mut inner = div().flex_col().gap_px(spacing);

        if config.full_width {
            inner = inner.w_full();
        } else if let Some(width) = config.width {
            inner = inner.w(width);
        }

        if let Some(max_width) = config.max_width {
            inner = inner.max_w(max_width);
        }

        if config.disabled {
            inner = inner.opacity(0.6);
        }

        for child in config.children {
            inner = inner.child(child);
        }

        Self { inner }
    }
}

impl Deref for Form {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Form {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ElementBuilder for Form {
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
pub(crate) struct FormConfig {
    pub(crate) spacing: Option<f32>,
    pub(crate) full_width: bool,
    pub(crate) disabled: bool,
    width: Option<f32>,
    max_width: Option<f32>,
    children: Vec<SharedElement>,
}

/// Builder for `Form`.
pub struct FormBuilder {
    pub(crate) config: FormConfig,
    built: OnceCell<Form>,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self {
            config: FormConfig::default(),
            built: OnceCell::new(),
        }
    }

    fn get_or_build(&self) -> &Form {
        self.built
            .get_or_init(|| Form::with_config(self.config.clone()))
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.config.spacing = Some(spacing);
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
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    pub fn child(mut self, child: impl ElementBuilder + 'static) -> Self {
        self.config.children.push(SharedElement(Arc::new(child)));
        self
    }

    pub fn build_component(self) -> Form {
        Form::with_config(self.config)
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementBuilder for FormBuilder {
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

pub fn form() -> FormBuilder {
    FormBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_builder_sets_spacing_and_full_width() {
        let form = form().spacing(20.0).w_full();
        assert_eq!(form.config.spacing, Some(20.0));
        assert!(form.config.full_width);
    }

    #[test]
    fn test_form_builder_sets_disabled() {
        let form = form().disabled(true);
        assert!(form.config.disabled);
    }
}
