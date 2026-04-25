//! Desktop event loop implementation using winit
//!
//! Supports multiple windows via `AppCommand::CreateWindow`.

use std::collections::HashMap;

use std::time::{Duration, Instant};

use crate::input;
use crate::window::DesktopWindow;
use blinc_platform::{
    current_ime_state, ControlFlow, Event, EventLoop, ImeCursorArea, ImeState, LifecycleEvent,
    PlatformError, Window, WindowConfig, WindowEvent, WindowId,
};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{StartCause, WindowEvent as WinitWindowEvent};
use winit::event_loop::{
    ActiveEventLoop, ControlFlow as WinitControlFlow, EventLoop as WinitEventLoop, EventLoopProxy,
};
use winit::keyboard::ModifiersState;
use winit::window::WindowId as WinitWindowId;

/// Commands sent to the event loop via `EventLoopProxy`.
///
/// Used for cross-thread requests that require access to `ActiveEventLoop`
/// (e.g., creating new windows, which can only happen inside the event handler).
#[derive(Debug)]
pub enum AppCommand {
    /// Wake the event loop (request redraw on all windows)
    Wake,
    /// Create a new window with the given configuration
    CreateWindow(WindowConfig),
    /// Close a specific window
    CloseWindow(WindowId),
}

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

trait DesktopImeWindow {
    fn set_ime_allowed(&self, allowed: bool);
    fn set_ime_cursor_area(&self, area: ImeCursorArea);
}

impl DesktopImeWindow for DesktopWindow {
    fn set_ime_allowed(&self, allowed: bool) {
        self.winit_window().set_ime_allowed(allowed);
    }

    fn set_ime_cursor_area(&self, area: ImeCursorArea) {
        self.winit_window().set_ime_cursor_area(
            LogicalPosition::new(area.x, area.y),
            LogicalSize::new(area.width, area.height),
        );
    }
}

fn sync_ime_window_state<W: DesktopImeWindow>(
    window: &W,
    applied: &mut ImeState,
    requested: ImeState,
) {
    if !requested.enabled && applied.cursor_area.is_some() {
        window.set_ime_cursor_area(ImeCursorArea::new(0.0, 0.0, 0.0, 0.0));
    }

    if applied.enabled != requested.enabled {
        window.set_ime_allowed(requested.enabled);
    }

    if requested.enabled && applied.cursor_area != requested.cursor_area {
        match requested.cursor_area {
            Some(area) => window.set_ime_cursor_area(area),
            None if applied.cursor_area.is_some() => {
                window.set_ime_cursor_area(ImeCursorArea::new(0.0, 0.0, 0.0, 0.0));
            }
            None => {}
        }
    }

    *applied = requested;
}

/// Proxy for waking up the event loop from another thread
///
/// Use this to request a redraw from a background animation thread,
/// or to send commands for window creation/destruction.
#[derive(Clone)]
pub struct WakeProxy {
    proxy: EventLoopProxy<AppCommand>,
}

impl WakeProxy {
    /// Wake up the event loop, causing it to process events and potentially redraw
    pub fn wake(&self) {
        let _ = self.proxy.send_event(AppCommand::Wake);
    }

    /// Request creation of a new window on the next event loop tick
    pub fn create_window(&self, config: WindowConfig) {
        let _ = self.proxy.send_event(AppCommand::CreateWindow(config));
    }

    /// Request closing a specific window
    pub fn close_window(&self, id: WindowId) {
        let _ = self.proxy.send_event(AppCommand::CloseWindow(id));
    }
}

/// Desktop event loop wrapping winit's event loop
pub struct DesktopEventLoop {
    event_loop: WinitEventLoop<AppCommand>,
    window_config: WindowConfig,
    wake_proxy: WakeProxy,
}

impl DesktopEventLoop {
    /// Create a new desktop event loop
    pub fn new(config: WindowConfig) -> Result<Self, PlatformError> {
        let event_loop = WinitEventLoop::with_user_event()
            .build()
            .map_err(|e| PlatformError::EventLoop(e.to_string()))?;

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
    /// This is useful for animation threads that need to request redraws,
    /// or for creating new windows from background tasks.
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

/// Convert a winit WindowId to our platform-agnostic WindowId
fn to_window_id(winit_id: WinitWindowId) -> WindowId {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    winit_id.hash(&mut hasher);
    WindowId(hasher.finish())
}

/// Internal winit application handler supporting multiple windows
struct DesktopApp<F>
where
    F: FnMut(Event, &DesktopWindow) -> ControlFlow,
{
    /// Config for the primary (initial) window
    window_config: WindowConfig,
    /// All open windows keyed by winit's WindowId
    windows: HashMap<WinitWindowId, DesktopWindow>,
    /// The primary window's winit ID (first window created)
    primary_winit_id: Option<WinitWindowId>,
    /// Event handler
    handler: F,
    /// Current keyboard modifiers
    modifiers: ModifiersState,
    /// Current mouse position (per-window tracking could be added later)
    mouse_position: (f32, f32),
    last_scroll_event_at: Option<Instant>,
    scroll_end_pending: bool,
    applied_ime_state: ImeState,
    /// Whether the app should exit
    should_exit: bool,
    /// Currently active modal window (blocks input to other windows)
    modal_window: Option<WinitWindowId>,
}

impl<F> DesktopApp<F>
where
    F: FnMut(Event, &DesktopWindow) -> ControlFlow,
{
    fn new(window_config: WindowConfig, handler: F) -> Self {
        Self {
            window_config,
            windows: HashMap::new(),
            primary_winit_id: None,
            handler,
            modifiers: ModifiersState::empty(),
            mouse_position: (0.0, 0.0),
            last_scroll_event_at: None,
            scroll_end_pending: false,
            applied_ime_state: ImeState::default(),
            should_exit: false,
            modal_window: None,
        }
    }

    /// Dispatch an event using the window identified by winit_id
    fn handle_event_for(&mut self, winit_id: WinitWindowId, event: Event) {
        if let Some(window) = self.windows.get(&winit_id) {
            let flow = (self.handler)(event, window);
            if flow == ControlFlow::Exit {
                self.should_exit = true;
            }
        }
        if let Some(window) = self.windows.get(&winit_id) {
            sync_ime_window_state(window, &mut self.applied_ime_state, current_ime_state());
        }
    }

    fn reset_scroll_state(&mut self) {
        self.last_scroll_event_at = None;
        self.scroll_end_pending = false;
    }

    /// Dispatch an event using the primary window (for global events)
    fn handle_event(&mut self, event: Event) {
        if let Some(primary_id) = self.primary_winit_id {
            self.handle_event_for(primary_id, event);
        }
    }

    /// Create a new window and register it
    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        config: &WindowConfig,
    ) -> Option<WinitWindowId> {
        match DesktopWindow::new(event_loop, config) {
            Ok(window) => {
                let winit_id = window.winit_window().id();
                self.windows.insert(winit_id, window);
                Some(winit_id)
            }
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                None
            }
        }
    }
}

impl<F> ApplicationHandler<AppCommand> for DesktopApp<F>
where
    F: FnMut(Event, &DesktopWindow) -> ControlFlow,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the primary window if we don't have one
        if self.primary_winit_id.is_none() {
            let config = self.window_config.clone();
            if let Some(winit_id) = self.create_window(event_loop, &config) {
                self.primary_winit_id = Some(winit_id);
                self.handle_event_for(winit_id, Event::Lifecycle(LifecycleEvent::Resumed));
            } else {
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.handle_event(Event::Lifecycle(LifecycleEvent::Suspended));
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        // Request redraw on wait timeout (frame tick) for all windows
        if matches!(cause, StartCause::WaitCancelled { .. } | StartCause::Poll) {
            for window in self.windows.values() {
                window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        winit_id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        // Block input to non-modal windows when a modal is active
        if let Some(modal_id) = self.modal_window {
            if winit_id != modal_id {
                // Allow close/resize/redraw but block input events
                match &event {
                    WinitWindowEvent::KeyboardInput { .. }
                    | WinitWindowEvent::MouseInput { .. }
                    | WinitWindowEvent::CursorMoved { .. }
                    | WinitWindowEvent::MouseWheel { .. }
                    | WinitWindowEvent::Touch(_) => return,
                    _ => {}
                }
            }
        }

        let wid = to_window_id(winit_id);

        match event {
            WinitWindowEvent::CloseRequested => {
                self.handle_event_for(winit_id, Event::Window(wid, WindowEvent::CloseRequested));

                // Remove the window
                self.windows.remove(&winit_id);

                // Clear modal if the closed window was the modal
                if self.modal_window == Some(winit_id) {
                    self.modal_window = None;
                }

                // If no windows remain, exit
                if self.windows.is_empty() {
                    self.should_exit = true;
                    event_loop.exit();
                }
            }

            WinitWindowEvent::Resized(size) => {
                self.handle_event_for(
                    winit_id,
                    Event::Window(
                        wid,
                        WindowEvent::Resized {
                            width: size.width,
                            height: size.height,
                        },
                    ),
                );
            }

            WinitWindowEvent::Moved(pos) => {
                self.handle_event_for(
                    winit_id,
                    Event::Window(wid, WindowEvent::Moved { x: pos.x, y: pos.y }),
                );
            }

            WinitWindowEvent::Focused(focused) => {
                if let Some(window) = self.windows.get(&winit_id) {
                    window.set_focused(focused);
                }
                self.handle_event_for(winit_id, Event::Window(wid, WindowEvent::Focused(focused)));
            }

            WinitWindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_event_for(
                    winit_id,
                    Event::Window(wid, WindowEvent::ScaleFactorChanged { scale_factor }),
                );
            }

            WinitWindowEvent::RedrawRequested => {
                self.handle_event_for(winit_id, Event::Frame(wid));
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
                self.handle_event_for(winit_id, Event::Input(wid, input_event));
                if let Some(window) = self.windows.get(&winit_id) {
                    window.request_redraw();
                }
            }

            WinitWindowEvent::Ime(ime) => {
                if let Some(input_event) = input::convert_ime_event(&ime) {
                    self.handle_event_for(winit_id, Event::Input(wid, input_event));
                }
                if let Some(window) = self.windows.get(&winit_id) {
                    window.request_redraw();
                }
            }

            WinitWindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
                let input_event = input::mouse_moved(self.mouse_position.0, self.mouse_position.1);
                self.handle_event_for(winit_id, Event::Input(wid, input_event));
            }

            WinitWindowEvent::MouseInput { state, button, .. } => {
                let (x, y) = self.mouse_position;
                let input_event = match state {
                    winit::event::ElementState::Pressed => input::mouse_pressed(button, x, y),
                    winit::event::ElementState::Released => input::mouse_released(button, x, y),
                };
                self.handle_event_for(winit_id, Event::Input(wid, input_event));
            }

            WinitWindowEvent::MouseWheel { delta, phase, .. } => {
                // `LineDelta` is emitted by classic step-wheel mice; each tick
                // is ±1 line. `PixelDelta` is emitted by trackpads / Magic Mouse
                // and already gives pixel-accurate scroll distance per event
                // (NSScrollView semantics on macOS, and equivalent on other
                // platforms). Line-based events need to be converted to pixels
                // using a per-line height — we use 40px to match what most
                // native apps feel like. Pixel deltas must be passed through
                // unchanged; a previous divide-by-10 here caused a perceptible
                // drag on trackpads (only 10 % of the intended scroll applied).
                const LINE_HEIGHT_PX: f32 = 40.0;
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        (x * LINE_HEIGHT_PX, y * LINE_HEIGHT_PX)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                let input_event = input::scroll_event(dx, dy, phase);
                self.handle_event_for(winit_id, Event::Input(wid, input_event));

                if matches!(
                    phase,
                    winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled
                ) {
                    self.handle_event_for(winit_id, Event::Input(wid, input::scroll_end_event()));
                    self.reset_scroll_state();
                } else {
                    self.last_scroll_event_at = Some(Instant::now());
                    self.scroll_end_pending = true;
                }
            }

            WinitWindowEvent::Touch(touch) => {
                let input_event = input::convert_touch_event(&touch);
                self.handle_event_for(winit_id, Event::Input(wid, input_event));
            }

            WinitWindowEvent::CursorEntered { .. } => {
                self.handle_event_for(
                    winit_id,
                    Event::Input(
                        wid,
                        blinc_platform::InputEvent::Mouse(blinc_platform::MouseEvent::Entered),
                    ),
                );
            }

            WinitWindowEvent::CursorLeft { .. } => {
                self.handle_event_for(
                    winit_id,
                    Event::Input(
                        wid,
                        blinc_platform::InputEvent::Mouse(blinc_platform::MouseEvent::Left),
                    ),
                );
            }

            WinitWindowEvent::DroppedFile(path) => {
                self.handle_event_for(
                    winit_id,
                    Event::Window(wid, WindowEvent::DroppedFile { paths: vec![path] }),
                );
            }

            WinitWindowEvent::HoveredFile(path) => {
                self.handle_event_for(
                    winit_id,
                    Event::Window(wid, WindowEvent::DroppedFileHovered { paths: vec![path] }),
                );
            }

            WinitWindowEvent::HoveredFileCancelled => {
                self.handle_event_for(
                    winit_id,
                    Event::Window(wid, WindowEvent::DroppedFileCancelled),
                );
            }

            WinitWindowEvent::PinchGesture { delta, phase, .. } => {
                let scroll_phase = match phase {
                    winit::event::TouchPhase::Started => blinc_platform::ScrollPhase::Started,
                    winit::event::TouchPhase::Moved => blinc_platform::ScrollPhase::Moved,
                    winit::event::TouchPhase::Ended => blinc_platform::ScrollPhase::Ended,
                    winit::event::TouchPhase::Cancelled => blinc_platform::ScrollPhase::Ended,
                };
                self.handle_event_for(
                    winit_id,
                    Event::Input(
                        wid,
                        blinc_platform::InputEvent::Pinch {
                            scale: 1.0 + delta as f32,
                            phase: scroll_phase,
                        },
                    ),
                );
            }

            WinitWindowEvent::RotationGesture { delta, phase, .. } => {
                let scroll_phase = match phase {
                    winit::event::TouchPhase::Started => blinc_platform::ScrollPhase::Started,
                    winit::event::TouchPhase::Moved => blinc_platform::ScrollPhase::Moved,
                    winit::event::TouchPhase::Ended => blinc_platform::ScrollPhase::Ended,
                    winit::event::TouchPhase::Cancelled => blinc_platform::ScrollPhase::Ended,
                };
                self.handle_event_for(
                    winit_id,
                    Event::Input(
                        wid,
                        blinc_platform::InputEvent::Rotation {
                            angle: delta.to_radians(),
                            phase: scroll_phase,
                        },
                    ),
                );
            }

            _ => {}
        }

        if self.should_exit {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let elapsed_since_last_scroll = self.last_scroll_event_at.map(|last| last.elapsed());
        if should_emit_synthetic_scroll_end(self.scroll_end_pending, elapsed_since_last_scroll) {
            if let Some(primary_id) = self.primary_winit_id {
                let wid = to_window_id(primary_id);
                self.handle_event_for(primary_id, Event::Input(wid, input::scroll_end_event()));
                if let Some(window) = self.windows.get(&primary_id) {
                    window.request_redraw();
                }
            }
            self.reset_scroll_state();
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

    fn user_event(&mut self, event_loop: &ActiveEventLoop, command: AppCommand) {
        match command {
            AppCommand::Wake => {
                // Wake from animation thread — request redraw on all windows
                for window in self.windows.values() {
                    window.request_redraw();
                }
            }
            AppCommand::CreateWindow(config) => {
                let is_modal = config.modal;
                if let Some(winit_id) = self.create_window(event_loop, &config) {
                    if is_modal {
                        self.modal_window = Some(winit_id);
                    }
                    let wid = to_window_id(winit_id);
                    self.handle_event_for(winit_id, Event::Lifecycle(LifecycleEvent::Resumed));
                    tracing::info!(
                        "Created new window {:?} (wid={:?}, modal={})",
                        winit_id,
                        wid,
                        is_modal
                    );
                }
            }
            AppCommand::CloseWindow(wid) => {
                // Find the winit ID for this WindowId
                let winit_id = self
                    .windows
                    .iter()
                    .find(|(_, w)| w.id() == wid)
                    .map(|(id, _)| *id);
                if let Some(winit_id) = winit_id {
                    self.handle_event_for(
                        winit_id,
                        Event::Window(wid, WindowEvent::CloseRequested),
                    );
                    self.windows.remove(&winit_id);

                    // Clear modal if the closed window was the modal
                    if self.modal_window == Some(winit_id) {
                        self.modal_window = None;
                    }

                    if self.windows.is_empty() {
                        self.should_exit = true;
                        event_loop.exit();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        scroll_end_deadline, should_emit_synthetic_scroll_end, sync_ime_window_state,
        DesktopImeWindow, SCROLL_END_DEBOUNCE,
    };
    use blinc_platform::{ImeCursorArea, ImeState};
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    #[derive(Default)]
    struct TestImeWindow {
        allowed: Mutex<Vec<bool>>,
        cursor_areas: Mutex<Vec<ImeCursorArea>>,
    }

    impl DesktopImeWindow for TestImeWindow {
        fn set_ime_allowed(&self, allowed: bool) {
            self.allowed.lock().expect("allowed lock").push(allowed);
        }

        fn set_ime_cursor_area(&self, area: ImeCursorArea) {
            self.cursor_areas
                .lock()
                .expect("cursor area lock")
                .push(area);
        }
    }

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

    #[test]
    fn sync_ime_window_state_updates_allowed_and_cursor_area() {
        let window = TestImeWindow::default();
        let mut applied = ImeState::default();
        let requested = ImeState {
            enabled: true,
            cursor_area: Some(ImeCursorArea::new(12.0, 24.0, 80.0, 30.0)),
            request: None,
        };

        sync_ime_window_state(&window, &mut applied, requested.clone());

        assert_eq!(*window.allowed.lock().expect("allowed lock"), vec![true]);
        assert_eq!(
            *window.cursor_areas.lock().expect("cursor area lock"),
            vec![ImeCursorArea::new(12.0, 24.0, 80.0, 30.0)]
        );
        assert_eq!(applied, requested);
    }

    #[test]
    fn sync_ime_window_state_clears_stale_cursor_area_when_requested_area_is_missing() {
        let window = TestImeWindow::default();
        let mut applied = ImeState {
            enabled: true,
            cursor_area: Some(ImeCursorArea::new(12.0, 24.0, 80.0, 30.0)),
            request: None,
        };

        sync_ime_window_state(
            &window,
            &mut applied,
            ImeState {
                enabled: true,
                cursor_area: None,
                request: None,
            },
        );

        assert_eq!(
            *window.allowed.lock().expect("allowed lock"),
            Vec::<bool>::new()
        );
        assert_eq!(
            *window.cursor_areas.lock().expect("cursor area lock"),
            vec![ImeCursorArea::new(0.0, 0.0, 0.0, 0.0)]
        );
        assert_eq!(
            applied,
            ImeState {
                enabled: true,
                cursor_area: None,
                request: None,
            }
        );
    }

    #[test]
    fn sync_ime_window_state_clears_cursor_area_before_reenabling_without_bounds() {
        let window = TestImeWindow::default();
        let mut applied = ImeState {
            enabled: true,
            cursor_area: Some(ImeCursorArea::new(12.0, 24.0, 80.0, 30.0)),
            request: None,
        };

        sync_ime_window_state(
            &window,
            &mut applied,
            ImeState {
                enabled: false,
                cursor_area: None,
                request: None,
            },
        );
        sync_ime_window_state(
            &window,
            &mut applied,
            ImeState {
                enabled: true,
                cursor_area: None,
                request: None,
            },
        );

        assert_eq!(
            *window.allowed.lock().expect("allowed lock"),
            vec![false, true]
        );
        assert_eq!(
            *window.cursor_areas.lock().expect("cursor area lock"),
            vec![ImeCursorArea::new(0.0, 0.0, 0.0, 0.0)]
        );
        assert_eq!(
            applied,
            ImeState {
                enabled: true,
                cursor_area: None,
                request: None,
            }
        );
    }
}
