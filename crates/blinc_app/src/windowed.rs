//! Windowed application runner
//!
//! Provides a unified API for running windowed Blinc applications across
//! desktop and Android platforms.
//!
//! # Example
//!
//! ```ignore
//! use blinc_app::prelude::*;
//! use blinc_app::windowed::WindowedApp;
//!
//! fn main() -> Result<()> {
//!     WindowedApp::run(WindowConfig::default(), |ctx| {
//!         // Build your UI using reactive signals
//!         let count = ctx.use_signal(0);
//!         let doubled = ctx.use_derived(move |cx| cx.get(count).unwrap_or(0) * 2);
//!
//!         div().w_full().h_full()
//!             .flex_center()
//!             .child(text(&format!("Count: {}", ctx.get(count).unwrap_or(0))).size(48.0))
//!     })
//! }
//! ```

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use blinc_core::reactive::{Derived, ReactiveGraph, Signal};
use blinc_layout::prelude::*;
use blinc_platform::{
    ControlFlow, Event, EventLoop, InputEvent, KeyState, LifecycleEvent, MouseEvent, Platform,
    TouchEvent, Window, WindowConfig, WindowEvent,
};

use crate::app::BlincApp;
use crate::error::{BlincError, Result};

#[cfg(all(feature = "windowed", not(target_os = "android")))]
use blinc_platform_desktop::DesktopPlatform;

/// Shared dirty flag type for element refs
pub type RefDirtyFlag = Arc<AtomicBool>;

/// Shared reactive graph for the application (thread-safe)
pub type SharedReactiveGraph = Arc<Mutex<ReactiveGraph>>;

/// Context passed to the UI builder function
pub struct WindowedContext {
    /// Current window width in physical pixels (matches surface size)
    pub width: f32,
    /// Current window height in physical pixels (matches surface size)
    pub height: f32,
    /// Current scale factor (physical / logical)
    pub scale_factor: f64,
    /// Whether the window is focused
    pub focused: bool,
    /// Event router for input event handling
    pub event_router: EventRouter,
    /// Shared dirty flag for element refs - when set, triggers UI rebuild
    ref_dirty_flag: RefDirtyFlag,
    /// Reactive graph for signal-based state management
    reactive: SharedReactiveGraph,
}

impl WindowedContext {
    fn from_window<W: Window>(
        window: &W,
        event_router: EventRouter,
        ref_dirty_flag: RefDirtyFlag,
        reactive: SharedReactiveGraph,
    ) -> Self {
        // Use physical size for rendering - the surface is in physical pixels
        // UI layout and rendering must use physical dimensions to match the surface
        let (width, height) = window.size();
        Self {
            width: width as f32,
            height: height as f32,
            scale_factor: window.scale_factor(),
            focused: window.is_focused(),
            event_router,
            ref_dirty_flag,
            reactive,
        }
    }

    /// Update context from window (preserving event router, dirty flag, and reactive graph)
    fn update_from_window<W: Window>(&mut self, window: &W) {
        let (width, height) = window.size();
        self.width = width as f32;
        self.height = height as f32;
        self.scale_factor = window.scale_factor();
        self.focused = window.is_focused();
    }

    // =========================================================================
    // Reactive Signal API
    // =========================================================================

    /// Create a new reactive signal with an initial value
    ///
    /// Signals are the core primitive of the reactive system. When a signal's
    /// value changes, any derived values or effects that depend on it will
    /// automatically update.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = ctx.use_signal(0);
    ///
    /// // In an event handler:
    /// ctx.set(count, ctx.get(count).unwrap_or(0) + 1);
    /// ```
    pub fn use_signal<T: Send + 'static>(&self, initial: T) -> Signal<T> {
        self.reactive.lock().unwrap().create_signal(initial)
    }

    /// Get the current value of a signal
    ///
    /// This automatically tracks the signal as a dependency when called
    /// within a derived computation or effect.
    pub fn get<T: Clone + 'static>(&self, signal: Signal<T>) -> Option<T> {
        self.reactive.lock().unwrap().get(signal)
    }

    /// Set the value of a signal, triggering reactive updates
    ///
    /// This will automatically trigger a UI rebuild.
    pub fn set<T: Send + 'static>(&self, signal: Signal<T>, value: T) {
        self.reactive.lock().unwrap().set(signal, value);
        // Mark dirty to trigger rebuild
        self.ref_dirty_flag.store(true, Ordering::SeqCst);
    }

    /// Update a signal using a function
    ///
    /// This is useful for incrementing counters or modifying state based
    /// on the current value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ctx.update(count, |n| n + 1);
    /// ```
    pub fn update<T: Clone + Send + 'static, F: FnOnce(T) -> T>(&self, signal: Signal<T>, f: F) {
        self.reactive.lock().unwrap().update(signal, f);
        self.ref_dirty_flag.store(true, Ordering::SeqCst);
    }

    /// Create a derived (computed) value
    ///
    /// Derived values are lazily computed and cached. They automatically
    /// track their signal dependencies and recompute when those signals change.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = ctx.use_signal(5);
    /// let doubled = ctx.use_derived(move |cx| cx.get(count).unwrap_or(0) * 2);
    ///
    /// assert_eq!(ctx.get_derived(doubled), Some(10));
    /// ```
    pub fn use_derived<T, F>(&self, compute: F) -> Derived<T>
    where
        T: Clone + Send + 'static,
        F: Fn(&ReactiveGraph) -> T + Send + 'static,
    {
        self.reactive.lock().unwrap().create_derived(compute)
    }

    /// Get the value of a derived computation
    pub fn get_derived<T: Clone + 'static>(&self, derived: Derived<T>) -> Option<T> {
        self.reactive.lock().unwrap().get_derived(derived)
    }

    /// Create an effect that runs when its dependencies change
    ///
    /// Effects are useful for side effects like logging, network requests,
    /// or syncing state with external systems.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = ctx.use_signal(0);
    ///
    /// ctx.use_effect(move |cx| {
    ///     let value = cx.get(count).unwrap_or(0);
    ///     println!("Count changed to: {}", value);
    /// });
    /// ```
    pub fn use_effect<F>(&self, run: F) -> blinc_core::reactive::Effect
    where
        F: FnMut(&ReactiveGraph) + Send + 'static,
    {
        self.reactive.lock().unwrap().create_effect(run)
    }

    /// Batch multiple signal updates into a single reactive update
    ///
    /// This is useful when updating multiple signals at once to avoid
    /// redundant recomputations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ctx.batch(|g| {
    ///     g.set(x, 10);
    ///     g.set(y, 20);
    ///     g.set(z, 30);
    /// });
    /// // Only one UI rebuild triggered
    /// ```
    pub fn batch<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ReactiveGraph) -> R,
    {
        let result = self.reactive.lock().unwrap().batch(f);
        self.ref_dirty_flag.store(true, Ordering::SeqCst);
        result
    }

    /// Get the shared reactive graph for advanced usage
    ///
    /// This is useful when you need to pass the graph to closures or
    /// store it for later use.
    pub fn reactive(&self) -> SharedReactiveGraph {
        Arc::clone(&self.reactive)
    }

    /// Create a new DivRef that will trigger rebuilds when modified
    ///
    /// Use this to create refs that can be mutated in event handlers.
    /// When you call `.borrow_mut()` or `.with_mut()` on the returned ref,
    /// the UI will automatically rebuild when the mutation completes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let card_ref = ctx.create_ref::<Div>();
    ///
    /// div()
    ///     .child(
    ///         div()
    ///             .bind(&card_ref)
    ///             .on_hover_enter({
    ///                 let r = card_ref.clone();
    ///                 move |_| {
    ///                     // This automatically triggers a rebuild
    ///                     r.with_mut(|d| *d = d.swap().bg(Color::RED));
    ///                 }
    ///             })
    ///     )
    /// ```
    pub fn create_ref<T>(&self) -> ElementRef<T> {
        ElementRef::with_dirty_flag(Arc::clone(&self.ref_dirty_flag))
    }

    /// Create a new DivRef (convenience method)
    pub fn div_ref(&self) -> DivRef {
        self.create_ref::<Div>()
    }

    /// Get the shared dirty flag for manual state management
    ///
    /// Use this when you want to create your own state types that trigger
    /// UI rebuilds when modified. When you modify state, set this flag to true.
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct MyState {
    ///     value: i32,
    ///     dirty_flag: RefDirtyFlag,
    /// }
    ///
    /// impl MyState {
    ///     fn set_value(&mut self, v: i32) {
    ///         self.value = v;
    ///         self.dirty_flag.store(true, Ordering::SeqCst);
    ///     }
    /// }
    /// ```
    pub fn dirty_flag(&self) -> RefDirtyFlag {
        Arc::clone(&self.ref_dirty_flag)
    }
}

/// Windowed application runner
///
/// Provides a simple way to run a Blinc application in a window
/// with automatic event handling and rendering.
pub struct WindowedApp;

impl WindowedApp {
    /// Initialize the platform asset loader
    ///
    /// On desktop, this sets up a filesystem-based loader.
    /// On Android, this would use the NDK AssetManager.
    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    fn init_asset_loader() {
        use blinc_platform::assets::{set_global_asset_loader, FilesystemAssetLoader};

        // Create a filesystem loader (uses current directory as base)
        let loader = FilesystemAssetLoader::new();

        // Try to set the global loader (ignore error if already set)
        let _ = set_global_asset_loader(Box::new(loader));
    }

    /// Run a windowed Blinc application on desktop platforms
    ///
    /// This is the main entry point for desktop applications. It creates
    /// a window, sets up GPU rendering, and runs the event loop.
    ///
    /// # Arguments
    ///
    /// * `config` - Window configuration (title, size, etc.)
    /// * `ui_builder` - Function that builds the UI tree given the window context
    ///
    /// # Example
    ///
    /// ```ignore
    /// WindowedApp::run(WindowConfig::default(), |ctx| {
    ///     div()
    ///         .w(ctx.width).h(ctx.height)
    ///         .bg([0.1, 0.1, 0.15, 1.0])
    ///         .flex_center()
    ///         .child(
    ///             div().glass().rounded(16.0).p(24.0)
    ///                 .child(text("Hello Blinc!").size(32.0))
    ///         )
    /// })
    /// ```
    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    pub fn run<F, E>(config: WindowConfig, ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder,
    {
        Self::run_desktop(config, ui_builder)
    }

    #[cfg(all(feature = "windowed", not(target_os = "android")))]
    fn run_desktop<F, E>(config: WindowConfig, mut ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder,
    {
        // Initialize the platform asset loader for cross-platform asset loading
        Self::init_asset_loader();

        let platform = DesktopPlatform::new().map_err(|e| BlincError::Platform(e.to_string()))?;
        let event_loop = platform
            .create_event_loop_with_config(config)
            .map_err(|e| BlincError::Platform(e.to_string()))?;

        // We need to defer BlincApp creation until we have a window
        let mut app: Option<BlincApp> = None;
        let mut surface: Option<wgpu::Surface<'static>> = None;
        let mut surface_config: Option<wgpu::SurfaceConfiguration> = None;

        // Persistent context with event router
        let mut ctx: Option<WindowedContext> = None;
        // Persistent render tree for hit testing and dirty tracking
        let mut render_tree: Option<RenderTree> = None;
        // Track if we need to rebuild UI (e.g., after resize)
        let mut needs_rebuild = true;
        // Shared dirty flag for element refs
        let ref_dirty_flag: RefDirtyFlag = Arc::new(AtomicBool::new(false));
        // Shared reactive graph for signal-based state management
        let reactive: SharedReactiveGraph = Arc::new(Mutex::new(ReactiveGraph::new()));

        event_loop
            .run(move |event, window| {
                match event {
                    Event::Lifecycle(LifecycleEvent::Resumed) => {
                        // Initialize GPU if not already done
                        if app.is_none() {
                            let winit_window = window.winit_window_arc();

                            match BlincApp::with_window(winit_window, None) {
                                Ok((blinc_app, surf)) => {
                                    let (width, height) = window.size();
                                    // Use the same texture format that the renderer's pipelines use
                                    let format = blinc_app.texture_format();
                                    let config = wgpu::SurfaceConfiguration {
                                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                        format,
                                        width,
                                        height,
                                        present_mode: wgpu::PresentMode::AutoVsync,
                                        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                                        view_formats: vec![],
                                        desired_maximum_frame_latency: 2,
                                    };
                                    surf.configure(&blinc_app.device(), &config);

                                    surface = Some(surf);
                                    surface_config = Some(config);
                                    app = Some(blinc_app);

                                    // Initialize context with event router, dirty flag, and reactive graph
                                    ctx = Some(WindowedContext::from_window(
                                        window,
                                        EventRouter::new(),
                                        Arc::clone(&ref_dirty_flag),
                                        Arc::clone(&reactive),
                                    ));

                                    tracing::info!("Blinc windowed app initialized");
                                }
                                Err(e) => {
                                    tracing::error!("Failed to initialize Blinc: {}", e);
                                    return ControlFlow::Exit;
                                }
                            }
                        }
                    }

                    Event::Window(WindowEvent::Resized { width, height }) => {
                        if let (Some(ref blinc_app), Some(ref surf), Some(ref mut config)) =
                            (&app, &surface, &mut surface_config)
                        {
                            if width > 0 && height > 0 {
                                config.width = width;
                                config.height = height;
                                surf.configure(&blinc_app.device(), config);
                                needs_rebuild = true;

                                // Dispatch RESIZE event to elements
                                if let (Some(ref mut windowed_ctx), Some(ref tree)) =
                                    (&mut ctx, &render_tree)
                                {
                                    windowed_ctx
                                        .event_router
                                        .on_window_resize(tree, width as f32, height as f32);
                                }
                            }
                        }
                    }

                    Event::Window(WindowEvent::Focused(focused)) => {
                        // Update context focus state
                        if let Some(ref mut windowed_ctx) = ctx {
                            windowed_ctx.focused = focused;

                            // Dispatch WINDOW_FOCUS or WINDOW_BLUR to the focused element
                            windowed_ctx.event_router.on_window_focus(focused);
                        }
                    }

                    Event::Window(WindowEvent::CloseRequested) => {
                        return ControlFlow::Exit;
                    }

                    // Handle input events
                    Event::Input(input_event) => {
                        // First phase: collect events using immutable borrow
                        let pending_events = if let (Some(ref mut windowed_ctx), Some(ref tree)) =
                            (&mut ctx, &render_tree)
                        {
                            let router = &mut windowed_ctx.event_router;

                            // Collect events from router
                            let mut pending_events: Vec<(LayoutNodeId, u32, f32, f32)> = Vec::new();

                            // Set up callback to collect events
                            router.set_event_callback({
                                let events = &mut pending_events as *mut Vec<(LayoutNodeId, u32, f32, f32)>;
                                move |node, event_type| {
                                    // SAFETY: This callback is only used within this scope
                                    unsafe {
                                        (*events).push((node, event_type, 0.0, 0.0));
                                    }
                                }
                            });

                            match input_event {
                                InputEvent::Mouse(mouse_event) => match mouse_event {
                                    MouseEvent::Moved { x, y } => {
                                        router.on_mouse_move(tree, x, y);
                                        for event in pending_events.iter_mut() {
                                            event.2 = x;
                                            event.3 = y;
                                        }
                                    }
                                    MouseEvent::ButtonPressed { button, x, y } => {
                                        let btn = convert_mouse_button(button);
                                        router.on_mouse_down(tree, x, y, btn);
                                        for event in pending_events.iter_mut() {
                                            event.2 = x;
                                            event.3 = y;
                                        }
                                    }
                                    MouseEvent::ButtonReleased { button, x, y } => {
                                        let btn = convert_mouse_button(button);
                                        router.on_mouse_up(tree, x, y, btn);
                                        for event in pending_events.iter_mut() {
                                            event.2 = x;
                                            event.3 = y;
                                        }
                                    }
                                    MouseEvent::Left => {
                                        router.on_mouse_leave();
                                    }
                                    MouseEvent::Entered => {
                                        let (mx, my) = router.mouse_position();
                                        router.on_mouse_move(tree, mx, my);
                                        for event in pending_events.iter_mut() {
                                            event.2 = mx;
                                            event.3 = my;
                                        }
                                    }
                                },
                                InputEvent::Keyboard(kb_event) => match kb_event.state {
                                    KeyState::Pressed => {
                                        router.on_key_down(0);
                                    }
                                    KeyState::Released => {
                                        router.on_key_up(0);
                                    }
                                },
                                InputEvent::Touch(touch_event) => match touch_event {
                                    TouchEvent::Started { x, y, .. } => {
                                        router.on_mouse_down(tree, x, y, MouseButton::Left);
                                        for event in pending_events.iter_mut() {
                                            event.2 = x;
                                            event.3 = y;
                                        }
                                    }
                                    TouchEvent::Moved { x, y, .. } => {
                                        router.on_mouse_move(tree, x, y);
                                        for event in pending_events.iter_mut() {
                                            event.2 = x;
                                            event.3 = y;
                                        }
                                    }
                                    TouchEvent::Ended { x, y, .. } => {
                                        router.on_mouse_up(tree, x, y, MouseButton::Left);
                                        for event in pending_events.iter_mut() {
                                            event.2 = x;
                                            event.3 = y;
                                        }
                                    }
                                    TouchEvent::Cancelled { .. } => {
                                        router.on_mouse_leave();
                                    }
                                },
                                InputEvent::Scroll { delta_x, delta_y } => {
                                    let (mx, my) = router.mouse_position();
                                    router.on_scroll(tree, delta_x, delta_y);
                                    for event in pending_events.iter_mut() {
                                        event.2 = mx;
                                        event.3 = my;
                                    }
                                }
                            }

                            router.clear_event_callback();
                            pending_events
                        } else {
                            Vec::new()
                        };

                        // Second phase: dispatch events with mutable borrow
                        // This automatically marks the tree dirty when handlers fire
                        if let Some(ref mut tree) = render_tree {
                            for (node, event_type, mouse_x, mouse_y) in pending_events {
                                tree.dispatch_event(node, event_type, mouse_x, mouse_y);
                            }
                        }
                    }

                    Event::Frame => {
                        if let (
                            Some(ref mut blinc_app),
                            Some(ref surf),
                            Some(ref config),
                            Some(ref mut windowed_ctx),
                        ) = (&mut app, &surface, &surface_config, &mut ctx)
                        {
                            // Get current frame
                            let frame = match surf.get_current_texture() {
                                Ok(f) => f,
                                Err(wgpu::SurfaceError::Lost) => {
                                    surf.configure(&blinc_app.device(), config);
                                    return ControlFlow::Continue;
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    tracing::error!("Out of GPU memory");
                                    return ControlFlow::Exit;
                                }
                                Err(e) => {
                                    tracing::warn!("Surface error: {:?}", e);
                                    return ControlFlow::Continue;
                                }
                            };

                            let view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());

                            // Update context from window
                            windowed_ctx.update_from_window(window);

                            // Check if event handlers marked anything dirty (auto-rebuild)
                            if let Some(ref tree) = render_tree {
                                if tree.needs_rebuild() {
                                    needs_rebuild = true;
                                }
                            }

                            // Check if element refs were modified (triggers rebuild)
                            if ref_dirty_flag.swap(false, Ordering::SeqCst) {
                                needs_rebuild = true;
                            }

                            // Build/rebuild render tree only when needed
                            // The tree persists across frames for stable node IDs and event handling
                            if needs_rebuild || render_tree.is_none() {
                                // Build UI and create render tree
                                let ui = ui_builder(windowed_ctx);
                                let mut tree = RenderTree::from_element(&ui);
                                tree.compute_layout(windowed_ctx.width, windowed_ctx.height);
                                render_tree = Some(tree);
                                needs_rebuild = false;
                            }

                            // Render from the cached tree
                            if let Some(ref tree) = render_tree {
                                if let Err(e) =
                                    blinc_app.render_tree(tree, &view, windowed_ctx.width as u32, windowed_ctx.height as u32)
                                {
                                    tracing::error!("Render error: {}", e);
                                }
                            }

                            frame.present();
                        }
                    }

                    _ => {}
                }

                ControlFlow::Continue
            })
            .map_err(|e| BlincError::Platform(e.to_string()))?;

        Ok(())
    }

    /// Placeholder for non-windowed builds
    #[cfg(not(feature = "windowed"))]
    pub fn run<F, E>(_config: WindowConfig, _ui_builder: F) -> Result<()>
    where
        F: FnMut(&mut WindowedContext) -> E + 'static,
        E: ElementBuilder,
    {
        Err(BlincError::Platform(
            "Windowed feature not enabled. Add 'windowed' feature to blinc_app".to_string(),
        ))
    }
}

/// Convert platform mouse button to layout mouse button
#[cfg(all(feature = "windowed", not(target_os = "android")))]
fn convert_mouse_button(button: blinc_platform::MouseButton) -> MouseButton {
    match button {
        blinc_platform::MouseButton::Left => MouseButton::Left,
        blinc_platform::MouseButton::Right => MouseButton::Right,
        blinc_platform::MouseButton::Middle => MouseButton::Middle,
        blinc_platform::MouseButton::Back => MouseButton::Back,
        blinc_platform::MouseButton::Forward => MouseButton::Forward,
        blinc_platform::MouseButton::Other(n) => MouseButton::Other(n),
    }
}

/// Convenience function to run a windowed app with default configuration
#[cfg(feature = "windowed")]
pub fn run_windowed<F, E>(ui_builder: F) -> Result<()>
where
    F: FnMut(&mut WindowedContext) -> E + 'static,
    E: ElementBuilder,
{
    WindowedApp::run(WindowConfig::default(), ui_builder)
}

/// Convenience function to run a windowed app with a title
#[cfg(feature = "windowed")]
pub fn run_windowed_with_title<F, E>(title: &str, ui_builder: F) -> Result<()>
where
    F: FnMut(&mut WindowedContext) -> E + 'static,
    E: ElementBuilder,
{
    let config = WindowConfig {
        title: title.to_string(),
        ..Default::default()
    };
    WindowedApp::run(config, ui_builder)
}
