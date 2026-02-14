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

    fn text_render_info(&self) -> Option<blinc_layout::div::TextRenderInfo> {
        self.0.text_render_info()
    }

    fn styled_text_render_info(&self) -> Option<blinc_layout::div::StyledTextRenderInfo> {
        self.0.styled_text_render_info()
    }

    fn svg_render_info(&self) -> Option<blinc_layout::div::SvgRenderInfo> {
        self.0.svg_render_info()
    }

    fn image_render_info(&self) -> Option<blinc_layout::div::ImageRenderInfo> {
        self.0.image_render_info()
    }

    fn canvas_render_info(&self) -> Option<blinc_layout::canvas::CanvasRenderFn> {
        self.0.canvas_render_info()
    }

    fn event_handlers(&self) -> Option<&blinc_layout::event_handler::EventHandlers> {
        self.0.event_handlers()
    }

    fn scroll_info(&self) -> Option<blinc_layout::scroll::ScrollRenderInfo> {
        self.0.scroll_info()
    }

    fn scroll_physics(&self) -> Option<blinc_layout::scroll::SharedScrollPhysics> {
        self.0.scroll_physics()
    }

    fn motion_animation_for_child(
        &self,
        child_index: usize,
    ) -> Option<blinc_layout::element::MotionAnimation> {
        self.0.motion_animation_for_child(child_index)
    }

    fn motion_bindings(&self) -> Option<blinc_layout::motion::MotionBindings> {
        self.0.motion_bindings()
    }

    fn motion_stable_id(&self) -> Option<&str> {
        self.0.motion_stable_id()
    }

    fn motion_should_replay(&self) -> bool {
        self.0.motion_should_replay()
    }

    fn motion_is_suspended(&self) -> bool {
        self.0.motion_is_suspended()
    }

    #[allow(deprecated)]
    fn motion_is_exiting(&self) -> bool {
        self.0.motion_is_exiting()
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        self.0.layout_style()
    }

    fn layout_bounds_storage(&self) -> Option<blinc_layout::renderer::LayoutBoundsStorage> {
        self.0.layout_bounds_storage()
    }

    fn layout_bounds_callback(&self) -> Option<blinc_layout::renderer::LayoutBoundsCallback> {
        self.0.layout_bounds_callback()
    }

    fn semantic_type_name(&self) -> Option<&'static str> {
        self.0.semantic_type_name()
    }

    fn element_id(&self) -> Option<&str> {
        self.0.element_id()
    }

    fn element_classes(&self) -> &[String] {
        self.0.element_classes()
    }

    fn bound_scroll_ref(&self) -> Option<&blinc_layout::selector::ScrollRef> {
        self.0.bound_scroll_ref()
    }

    fn motion_on_ready_callback(
        &self,
    ) -> Option<std::sync::Arc<dyn Fn(blinc_layout::element::ElementBounds) + Send + Sync>> {
        self.0.motion_on_ready_callback()
    }

    fn layout_animation_config(
        &self,
    ) -> Option<blinc_layout::layout_animation::LayoutAnimationConfig> {
        self.0.layout_animation_config()
    }

    fn visual_animation_config(
        &self,
    ) -> Option<blinc_layout::visual_animation::VisualAnimationConfig> {
        self.0.visual_animation_config()
    }
}
