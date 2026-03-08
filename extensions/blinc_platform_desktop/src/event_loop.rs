//! Desktop event loop implementation using winit

use std::time::{Duration, Instant};

use crate::input;
use crate::window::DesktopWindow;
use blinc_platform::{
    ControlFlow, Event, EventLoop, LifecycleEvent, PlatformError, Window, WindowConfig, WindowEvent,
};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent as WinitWindowEvent};
use winit::event_loop::{
    ActiveEventLoop, ControlFlow as WinitControlFlow, EventLoop as WinitEventLoop, EventLoopProxy,
};
use winit::keyboard::ModifiersState;
use winit::window::WindowId;

// If the platform doesn't send `TouchPhase::Ended` promptly for wheel/trackpad,
// synthesize `ScrollEnd` after a short inactivity window.
const SCROLL_END_DEBOUNCE: Duration = Duration::from_millis(36);

fn should_emit_synthetic_scroll_end(
    scroll_end_pending: bool,
    elapsed_since_last_scroll: Option<Duration>,
) -> bool {
    scroll_end_pending
        && elapsed_since_last_scroll.is_some_and(|elapsed| elapsed >= SCROLL_END_DEBOUNCE)
}

fn scroll_end_deadline(
    scroll_end_pending: bool,
    last_scroll_event_at: Option<Instant>,
) -> Option<Instant> {
    if !scroll_end_pending {
        return None;
    }
    last_scroll_event_at.map(|last| last + SCROLL_END_DEBOUNCE)
}

/// Proxy for waking up the event loop from another thread
///
/// Use this to request a redraw from a background animation thread.
/// Call `wake()` to send a wake-up signal to the event loop.
#[derive(Clone)]
pub struct WakeProxy {
    proxy: EventLoopProxy<()>,
}

impl WakeProxy {
    /// Wake up the event loop, causing it to process events and potentially redraw
    pub fn wake(&self) {
        // Ignore errors (e.g., if event loop has exited)
        let _ = self.proxy.send_event(());
    }
}

/// Desktop event loop wrapping winit's event loop
pub struct DesktopEventLoop {
    event_loop: WinitEventLoop<()>,
    window_config: WindowConfig,
    wake_proxy: WakeProxy,
}

impl DesktopEventLoop {
    /// Create a new desktop event loop
    pub fn new(config: WindowConfig) -> Result<Self, PlatformError> {
        // NOTE(macos): Explicitly set activation policy to Regular so the window behaves like a
        // normal app window (shows up in window lists, focus/activation works, and automation
        // tools can detect it). Without this, non-bundled binaries can behave like UI-less helpers.
        let event_loop = {
            let mut builder = WinitEventLoop::builder();

            #[cfg(target_os = "macos")]
            {
                use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
                builder.with_activation_policy(ActivationPolicy::Regular);
            }

            builder
                .build()
                .map_err(|e| PlatformError::EventLoop(e.to_string()))?
        };

        let wake_proxy = WakeProxy {
            proxy: event_loop.create_proxy(),
        };

        Ok(Self {
            event_loop,
            window_config: config,
            wake_proxy,
        })
    }

    /// Get a wake proxy that can be used to wake up the event loop from another thread
    ///
    /// This is useful for animation threads that need to request redraws.
    pub fn wake_proxy(&self) -> WakeProxy {
        self.wake_proxy.clone()
    }
}

impl EventLoop for DesktopEventLoop {
    type Window = DesktopWindow;

    fn run<F>(self, handler: F) -> Result<(), PlatformError>
    where
        F: FnMut(Event, &Self::Window) -> ControlFlow + 'static,
    {
        let mut app = DesktopApp::new(self.window_config, handler);
        self.event_loop
            .run_app(&mut app)
            .map_err(|e| PlatformError::EventLoop(e.to_string()))
    }
}

/// Internal winit application handler
struct DesktopApp<F>
where
    F: FnMut(Event, &DesktopWindow) -> ControlFlow,
{
    window_config: WindowConfig,
    window: Option<DesktopWindow>,
    handler: F,
    modifiers: ModifiersState,
    mouse_position: (f32, f32),
    last_scroll_event_at: Option<Instant>,
    scroll_end_pending: bool,
    should_exit: bool,
}

impl<F> DesktopApp<F>
where
    F: FnMut(Event, &DesktopWindow) -> ControlFlow,
{
    fn new(window_config: WindowConfig, handler: F) -> Self {
        Self {
            window_config,
            window: None,
            handler,
            modifiers: ModifiersState::empty(),
            mouse_position: (0.0, 0.0),
            last_scroll_event_at: None,
            scroll_end_pending: false,
            should_exit: false,
        }
    }

    fn handle_event(&mut self, event: Event) {
        if let Some(ref window) = self.window {
            let flow = (self.handler)(event, window);
            if flow == ControlFlow::Exit {
                self.should_exit = true;
            }
        }
    }

    fn reset_scroll_state(&mut self) {
        self.last_scroll_event_at = None;
        self.scroll_end_pending = false;
    }
}

impl<F> ApplicationHandler for DesktopApp<F>
where
    F: FnMut(Event, &DesktopWindow) -> ControlFlow,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window if we don't have one
        if self.window.is_none() {
            match DesktopWindow::new(event_loop, &self.window_config) {
                Ok(window) => {
                    self.window = Some(window);
                    self.handle_event(Event::Lifecycle(LifecycleEvent::Resumed));
                }
                Err(e) => {
                    tracing::error!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.handle_event(Event::Lifecycle(LifecycleEvent::Suspended));
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        // Request redraw on wait timeout (frame tick)
        if matches!(cause, StartCause::WaitCancelled { .. } | StartCause::Poll) {
            if let Some(ref window) = self.window {
                window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WinitWindowEvent,
    ) {
        match event {
            WinitWindowEvent::CloseRequested => {
                self.handle_event(Event::Window(WindowEvent::CloseRequested));
                if self.should_exit {
                    event_loop.exit();
                }
            }

            WinitWindowEvent::Resized(size) => {
                self.handle_event(Event::Window(WindowEvent::Resized {
                    width: size.width,
                    height: size.height,
                }));
            }

            WinitWindowEvent::Moved(pos) => {
                self.handle_event(Event::Window(WindowEvent::Moved { x: pos.x, y: pos.y }));
            }

            WinitWindowEvent::Focused(focused) => {
                if let Some(ref window) = self.window {
                    window.set_focused(focused);
                }
                self.handle_event(Event::Window(WindowEvent::Focused(focused)));
            }

            WinitWindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_event(Event::Window(WindowEvent::ScaleFactorChanged {
                    scale_factor,
                }));
            }

            WinitWindowEvent::RedrawRequested => {
                self.handle_event(Event::Frame);
                if self.should_exit {
                    event_loop.exit();
                }
            }

            WinitWindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }

            WinitWindowEvent::KeyboardInput { event, .. } => {
                let input_event =
                    input::convert_keyboard_event(&event.logical_key, event.state, self.modifiers);
                self.handle_event(Event::Input(input_event));
                // Request immediate redraw so text input changes render instantly
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            WinitWindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
                let input_event = input::mouse_moved(self.mouse_position.0, self.mouse_position.1);
                self.handle_event(Event::Input(input_event));
            }

            WinitWindowEvent::MouseInput { state, button, .. } => {
                let (x, y) = self.mouse_position;
                let input_event = match state {
                    winit::event::ElementState::Pressed => input::mouse_pressed(button, x, y),
                    winit::event::ElementState::Released => input::mouse_released(button, x, y),
                };
                self.handle_event(Event::Input(input_event));
            }

            WinitWindowEvent::MouseWheel { delta, phase, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        (pos.x as f32 / 10.0, pos.y as f32 / 10.0)
                    }
                };
                let input_event = input::scroll_event(dx, dy, phase);
                self.handle_event(Event::Input(input_event));
                self.last_scroll_event_at = Some(Instant::now());
                self.scroll_end_pending = true;

                // If scroll gesture ended or momentum ended, send a scroll end event
                if matches!(
                    phase,
                    winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled
                ) {
                    self.handle_event(Event::Input(input::scroll_end_event()));
                    self.reset_scroll_state();
                }
            }

            // Trackpad pinch-to-zoom gesture (macOS/iOS)
            WinitWindowEvent::PinchGesture { delta, phase, .. } => {
                // winit provides a magnification delta. Convert to ratio scale delta:
                // 0.0 -> 1.0 (no change), 0.1 -> 1.1 (zoom in), -0.1 -> 0.9 (zoom out).
                // Clamp to avoid negative/zero scales on weird driver values.
                let scale = (1.0_f32 + delta as f32).clamp(0.01, 100.0);

                // Only emit updates while the gesture is active.
                if matches!(
                    phase,
                    winit::event::TouchPhase::Started | winit::event::TouchPhase::Moved
                ) {
                    self.handle_event(Event::Input(input::pinch_event(scale)));
                }
            }

            WinitWindowEvent::Touch(touch) => {
                let input_event = input::convert_touch_event(&touch);
                self.handle_event(Event::Input(input_event));
            }

            WinitWindowEvent::CursorEntered { .. } => {
                self.handle_event(Event::Input(blinc_platform::InputEvent::Mouse(
                    blinc_platform::MouseEvent::Entered,
                )));
            }

            WinitWindowEvent::CursorLeft { .. } => {
                self.handle_event(Event::Input(blinc_platform::InputEvent::Mouse(
                    blinc_platform::MouseEvent::Left,
                )));
            }

            _ => {}
        }

        // Check for exit
        if self.should_exit {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let elapsed_since_last_scroll = self.last_scroll_event_at.map(|last| last.elapsed());
        if should_emit_synthetic_scroll_end(self.scroll_end_pending, elapsed_since_last_scroll) {
            self.handle_event(Event::Input(input::scroll_end_event()));
            self.reset_scroll_state();
            if let Some(ref window) = self.window {
                window.request_redraw();
            }
            return;
        }

        if let Some(deadline) =
            scroll_end_deadline(self.scroll_end_pending, self.last_scroll_event_at)
        {
            event_loop.set_control_flow(WinitControlFlow::WaitUntil(deadline));
        }
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.handle_event(Event::Lifecycle(LifecycleEvent::LowMemory));
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        // Wake event from animation thread - request a redraw
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{scroll_end_deadline, should_emit_synthetic_scroll_end, SCROLL_END_DEBOUNCE};
    use std::time::{Duration, Instant};

    #[test]
    fn synthetic_scroll_end_requires_pending_and_elapsed_threshold() {
        assert!(!should_emit_synthetic_scroll_end(
            false,
            Some(SCROLL_END_DEBOUNCE)
        ));
        assert!(!should_emit_synthetic_scroll_end(true, None));
        assert!(!should_emit_synthetic_scroll_end(
            true,
            Some(SCROLL_END_DEBOUNCE - Duration::from_millis(1))
        ));
        assert!(should_emit_synthetic_scroll_end(
            true,
            Some(SCROLL_END_DEBOUNCE)
        ));
        assert!(should_emit_synthetic_scroll_end(
            true,
            Some(SCROLL_END_DEBOUNCE + Duration::from_millis(1))
        ));
    }

    #[test]
    fn scroll_end_deadline_requires_pending_and_timestamp() {
        let now = Instant::now();
        assert_eq!(scroll_end_deadline(false, Some(now)), None);
        assert_eq!(scroll_end_deadline(true, None), None);
        assert_eq!(
            scroll_end_deadline(true, Some(now)),
            Some(now + SCROLL_END_DEBOUNCE)
        );
    }
}
