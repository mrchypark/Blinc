use std::sync::Arc;

use blinc_layout::div::ElementTypeId;
use blinc_layout::prelude::ElementBuilder;

#[derive(Clone)]
pub(crate) struct SharedElement(Arc<dyn ElementBuilder>);

impl SharedElement {
    pub(crate) fn new(child: impl ElementBuilder + 'static) -> Self {
        Self(Arc::new(child))
    }
}

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
