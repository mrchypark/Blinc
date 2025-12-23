//! Button widget with FSM-driven interactions
//!
//! The Button widget provides:
//! - Visual states: idle, hovered, pressed, disabled
//! - FSM-driven state transitions
//! - Spring physics animations for scale/color
//! - Customizable appearance

use blinc_animation::spring::{Spring, SpringConfig};
use blinc_core::events::{event_types, Event};
use blinc_core::fsm::StateMachine;
use blinc_core::{Color, Shadow, Transform};
use blinc_layout::prelude::*;

use crate::context::WidgetContext;
use crate::widget::WidgetId;

/// Button states
pub mod states {
    pub const IDLE: u32 = 0;
    pub const HOVERED: u32 = 1;
    pub const PRESSED: u32 = 2;
    pub const DISABLED: u32 = 3;
}

/// Button configuration
#[derive(Clone)]
pub struct ButtonConfig {
    /// Button label text
    pub label: String,
    /// Base background color
    pub bg_color: Color,
    /// Hover background color
    pub hover_color: Color,
    /// Pressed background color
    pub pressed_color: Color,
    /// Disabled background color
    pub disabled_color: Color,
    /// Text color
    pub text_color: Color,
    /// Disabled text color
    pub disabled_text_color: Color,
    /// Font size
    pub font_size: f32,
    /// Corner radius
    pub corner_radius: f32,
    /// Padding (horizontal, vertical)
    pub padding: (f32, f32),
    /// Whether the button is disabled
    pub disabled: bool,
    /// Scale when pressed
    pub pressed_scale: f32,
    /// Shadow style
    pub shadow: Option<Shadow>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            bg_color: Color::rgba(0.2, 0.5, 0.9, 1.0),
            hover_color: Color::rgba(0.3, 0.6, 1.0, 1.0),
            pressed_color: Color::rgba(0.15, 0.4, 0.8, 1.0),
            disabled_color: Color::rgba(0.5, 0.5, 0.5, 0.5),
            text_color: Color::WHITE,
            disabled_text_color: Color::rgba(0.7, 0.7, 0.7, 1.0),
            font_size: 16.0,
            corner_radius: 8.0,
            padding: (16.0, 8.0),
            disabled: false,
            pressed_scale: 0.97,
            shadow: Some(Shadow::new(0.0, 2.0, 4.0, Color::rgba(0.0, 0.0, 0.0, 0.2))),
        }
    }
}

impl ButtonConfig {
    /// Create a new button config with a label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    /// Set the background color
    pub fn bg(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Set the hover color
    pub fn hover(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }

    /// Set the pressed color
    pub fn pressed(mut self, color: Color) -> Self {
        self.pressed_color = color;
        self
    }

    /// Set the text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set the font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set the padding
    pub fn padding(mut self, h: f32, v: f32) -> Self {
        self.padding = (h, v);
        self
    }

    /// Set whether the button is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the shadow
    pub fn shadow(mut self, shadow: Option<Shadow>) -> Self {
        self.shadow = shadow;
        self
    }
}

/// Button widget state
pub struct ButtonState {
    /// Current visual scale (animated)
    pub scale: f32,
    /// Current background color (animated)
    pub current_bg: Color,
    /// Spring for scale animation
    scale_spring: Spring,
    /// Whether the button was clicked (cleared after reading)
    clicked: bool,
}

impl Clone for ButtonState {
    fn clone(&self) -> Self {
        Self {
            scale: self.scale,
            current_bg: self.current_bg,
            // Create a new spring at the same position (can't clone springs)
            scale_spring: Spring::new(SpringConfig::snappy(), self.scale),
            clicked: self.clicked,
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            scale: 1.0,
            current_bg: Color::rgba(0.2, 0.5, 0.9, 1.0),
            scale_spring: Spring::new(SpringConfig::snappy(), 1.0),
            clicked: false,
        }
    }
}

impl ButtonState {
    /// Create a new button state with a config
    pub fn new(config: &ButtonConfig) -> Self {
        Self {
            scale: 1.0,
            current_bg: config.bg_color,
            scale_spring: Spring::new(SpringConfig::snappy(), 1.0),
            clicked: false,
        }
    }

    /// Update animations (call each frame)
    pub fn update(&mut self, dt: f32) {
        self.scale_spring.step(dt);
        self.scale = self.scale_spring.value();
    }

    /// Check if was clicked and clear the flag
    pub fn take_clicked(&mut self) -> bool {
        std::mem::take(&mut self.clicked)
    }

    /// Set the scale target
    pub fn set_scale_target(&mut self, target: f32) {
        self.scale_spring.set_target(target);
    }
}

/// Button widget
pub struct Button {
    /// Widget ID
    id: WidgetId,
    /// Configuration
    config: ButtonConfig,
    /// Click callback
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl Button {
    /// Create a new button
    pub fn new(ctx: &mut WidgetContext, label: impl Into<String>) -> Self {
        let config = ButtonConfig::new(label);
        Self::with_config(ctx, config)
    }

    /// Create a button with custom config
    pub fn with_config(ctx: &mut WidgetContext, config: ButtonConfig) -> Self {
        let fsm = Self::create_fsm(&config);
        let id = ctx.register_widget_with_fsm(fsm);

        // Initialize button state
        let state = ButtonState::new(&config);
        ctx.set_widget_state(id, state);

        Self {
            id,
            config,
            on_click: None,
        }
    }

    /// Create the button FSM
    fn create_fsm(config: &ButtonConfig) -> StateMachine {
        if config.disabled {
            // Disabled button has no transitions
            StateMachine::builder(states::DISABLED).build()
        } else {
            StateMachine::builder(states::IDLE)
                .on(states::IDLE, event_types::POINTER_ENTER, states::HOVERED)
                .on(states::HOVERED, event_types::POINTER_LEAVE, states::IDLE)
                .on(states::HOVERED, event_types::POINTER_DOWN, states::PRESSED)
                .on(states::PRESSED, event_types::POINTER_UP, states::HOVERED)
                .on(states::PRESSED, event_types::POINTER_LEAVE, states::IDLE)
                .build()
        }
    }

    /// Get the widget ID
    pub fn id(&self) -> WidgetId {
        self.id
    }

    /// Set the click callback
    pub fn on_click<F: FnMut() + Send + 'static>(mut self, callback: F) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }

    /// Handle an event
    pub fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) {
        if self.config.disabled {
            return;
        }

        let old_state = ctx.get_fsm_state(self.id).unwrap_or(states::IDLE);

        // Dispatch to FSM
        ctx.dispatch_event(self.id, event);

        let new_state = ctx.get_fsm_state(self.id).unwrap_or(states::IDLE);

        // Update visual state based on FSM state
        if let Some(state) = ctx.get_widget_state_mut::<ButtonState>(self.id) {
            match new_state {
                states::IDLE => {
                    state.set_scale_target(1.0);
                    state.current_bg = self.config.bg_color;
                }
                states::HOVERED => {
                    state.set_scale_target(1.0);
                    state.current_bg = self.config.hover_color;
                }
                states::PRESSED => {
                    state.set_scale_target(self.config.pressed_scale);
                    state.current_bg = self.config.pressed_color;
                }
                states::DISABLED => {
                    state.set_scale_target(1.0);
                    state.current_bg = self.config.disabled_color;
                }
                _ => {}
            }

            // Detect click (transition from PRESSED to HOVERED on pointer up)
            if old_state == states::PRESSED && new_state == states::HOVERED {
                state.clicked = true;
                if let Some(ref mut callback) = self.on_click {
                    callback();
                }
            }
        }
    }

    /// Update animations (call each frame)
    pub fn update(&self, ctx: &mut WidgetContext, dt: f32) {
        if let Some(state) = ctx.get_widget_state_mut::<ButtonState>(self.id) {
            let old_scale = state.scale;
            state.update(dt);

            // Mark dirty if scale changed significantly
            if (state.scale - old_scale).abs() > 0.001 {
                ctx.mark_dirty(self.id);
            }
        }
    }

    /// Check if the button was clicked (and clear the flag)
    pub fn was_clicked(&self, ctx: &mut WidgetContext) -> bool {
        ctx.get_widget_state_mut::<ButtonState>(self.id)
            .map(|s| s.take_clicked())
            .unwrap_or(false)
    }

    /// Build the button's UI element
    pub fn build(&self, ctx: &WidgetContext) -> Div {
        let state = ctx
            .get_widget_state::<ButtonState>(self.id)
            .cloned()
            .unwrap_or_default();

        let fsm_state = ctx.get_fsm_state(self.id).unwrap_or(states::IDLE);

        let (bg_color, text_color) = if fsm_state == states::DISABLED {
            (self.config.disabled_color, self.config.disabled_text_color)
        } else {
            (state.current_bg, self.config.text_color)
        };

        let mut button = div()
            .px(self.config.padding.0)
            .py(self.config.padding.1)
            .bg(bg_color.into())
            .rounded(self.config.corner_radius)
            .items_center()
            .justify_center()
            .transform(Transform::scale(state.scale, state.scale))
            .child(
                text(&self.config.label)
                    .size(self.config.font_size)
                    .color(text_color.into()),
            );

        if let Some(ref shadow) = self.config.shadow {
            button = button.shadow(shadow.clone());
        }

        button
    }
}

/// Create a button with a label
pub fn button(label: impl Into<String>) -> ButtonBuilder {
    ButtonBuilder {
        config: ButtonConfig::new(label),
        on_click: None,
    }
}

/// Builder for creating buttons
pub struct ButtonBuilder {
    config: ButtonConfig,
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl ButtonBuilder {
    /// Set the background color
    pub fn bg(mut self, color: impl Into<Color>) -> Self {
        self.config.bg_color = color.into();
        self
    }

    /// Set the hover color
    pub fn hover(mut self, color: impl Into<Color>) -> Self {
        self.config.hover_color = color.into();
        self
    }

    /// Set the pressed color
    pub fn pressed(mut self, color: impl Into<Color>) -> Self {
        self.config.pressed_color = color.into();
        self
    }

    /// Set the text color
    pub fn text_color(mut self, color: impl Into<Color>) -> Self {
        self.config.text_color = color.into();
        self
    }

    /// Set the font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.config.font_size = size;
        self
    }

    /// Set the corner radius
    pub fn rounded(mut self, radius: f32) -> Self {
        self.config.corner_radius = radius;
        self
    }

    /// Set the padding
    pub fn padding(mut self, h: f32, v: f32) -> Self {
        self.config.padding = (h, v);
        self
    }

    /// Set whether the button is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// Set the shadow
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.config.shadow = Some(shadow);
        self
    }

    /// Remove shadow
    pub fn no_shadow(mut self) -> Self {
        self.config.shadow = None;
        self
    }

    /// Set the click callback
    pub fn on_click<F: FnMut() + Send + 'static>(mut self, callback: F) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }

    /// Build the button widget
    pub fn build(self, ctx: &mut WidgetContext) -> Button {
        let mut button = Button::with_config(ctx, self.config);
        button.on_click = self.on_click;
        button
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_core::events::EventData;

    fn make_event(event_type: u32) -> Event {
        Event {
            event_type,
            target: 0,
            data: EventData::Pointer {
                x: 0.0,
                y: 0.0,
                button: 0,
                pressure: 1.0,
            },
            timestamp: 0,
            propagation_stopped: false,
        }
    }

    #[test]
    fn test_button_creation() {
        let mut ctx = WidgetContext::new();
        let button = Button::new(&mut ctx, "Click me");

        assert!(ctx.is_registered(button.id()));
        assert_eq!(ctx.get_fsm_state(button.id()), Some(states::IDLE));
    }

    #[test]
    fn test_button_state_transitions() {
        let mut ctx = WidgetContext::new();
        let mut button = Button::new(&mut ctx, "Click me");

        // Simulate hover
        let hover_event = make_event(event_types::POINTER_ENTER);
        button.handle_event(&mut ctx, &hover_event);
        assert_eq!(ctx.get_fsm_state(button.id()), Some(states::HOVERED));

        // Simulate press
        let press_event = make_event(event_types::POINTER_DOWN);
        button.handle_event(&mut ctx, &press_event);
        assert_eq!(ctx.get_fsm_state(button.id()), Some(states::PRESSED));

        // Simulate release (click)
        let release_event = make_event(event_types::POINTER_UP);
        button.handle_event(&mut ctx, &release_event);
        assert_eq!(ctx.get_fsm_state(button.id()), Some(states::HOVERED));

        // Check clicked flag
        assert!(button.was_clicked(&mut ctx));
        // Flag should be cleared after reading
        assert!(!button.was_clicked(&mut ctx));
    }

    #[test]
    fn test_disabled_button() {
        let mut ctx = WidgetContext::new();
        let config = ButtonConfig::new("Disabled").disabled(true);
        let button = Button::with_config(&mut ctx, config);

        assert_eq!(ctx.get_fsm_state(button.id()), Some(states::DISABLED));
    }
}
