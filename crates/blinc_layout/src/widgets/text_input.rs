//! Ready-to-use TextInput widget
//!
//! Single-line text input with:
//! - Visual states: idle, hovered, focused (via FSM-driven Stateful)
//! - Cursor blinking via AnimatedValue + Canvas (no rebuilds)
//! - Incremental updates: prop updates for visuals, subtree rebuilds for content
//! - No full UI rebuilds - uses queue_prop_update and queue_subtree_rebuild
//!
//! # Example
//!
//! ```ignore
//! let input_data = text_input_data_with_placeholder("Enter username");
//! text_input(&input_data)
//!     .w(280.0)
//!     .rounded(12.0)
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use blinc_core::Color;

use crate::canvas::canvas;
use crate::div::{div, Div, ElementBuilder};
use crate::element::RenderProps;
use crate::stateful::{refresh_stateful, SharedState, StatefulInner, StateTransitions, Stateful, TextFieldState};
use crate::text::text;
use crate::text_selection::{clear_selection, set_selection, SelectionSource};
use crate::tree::{LayoutNodeId, LayoutTree};
use crate::widgets::cursor::{cursor_state, CursorAnimation, SharedCursorState};

/// Get elapsed time in milliseconds since app start (for cursor blinking)
pub fn elapsed_ms() -> u64 {
    static START_TIME: OnceLock<std::time::Instant> = OnceLock::new();
    let start = START_TIME.get_or_init(std::time::Instant::now);
    start.elapsed().as_millis() as u64
}

/// Standard cursor blink interval in milliseconds
pub const CURSOR_BLINK_INTERVAL_MS: u64 = 400;

// =============================================================================
// Global focus tracking
// =============================================================================

static GLOBAL_FOCUS_COUNT: AtomicU64 = AtomicU64::new(0);
static NEEDS_REBUILD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static NEEDS_CONTINUOUS_REDRAW: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static FOCUSED_TEXT_INPUT: Mutex<Option<Weak<Mutex<TextInputData>>>> = Mutex::new(None);
static FOCUSED_TEXT_AREA: Mutex<Option<Weak<Mutex<crate::widgets::text_area::TextAreaState>>>> =
    Mutex::new(None);

/// Callback for setting continuous redraw on the animation scheduler
/// This is set by the windowed app to bridge text widgets with the animation system
static CONTINUOUS_REDRAW_CALLBACK: Mutex<Option<Box<dyn Fn(bool) + Send + Sync>>> =
    Mutex::new(None);

/// Set the callback for continuous redraw requests
///
/// This should be called once during app initialization to connect
/// text widget focus tracking with the animation scheduler.
pub fn set_continuous_redraw_callback<F>(callback: F)
where
    F: Fn(bool) + Send + Sync + 'static,
{
    let mut guard = CONTINUOUS_REDRAW_CALLBACK.lock().unwrap();
    *guard = Some(Box::new(callback));
}

/// Internal function to notify animation scheduler about cursor animation needs
fn notify_continuous_redraw(enabled: bool) {
    if let Ok(guard) = CONTINUOUS_REDRAW_CALLBACK.lock() {
        if let Some(ref callback) = *guard {
            callback(enabled);
        }
    }
}

pub fn has_focused_text_input() -> bool {
    GLOBAL_FOCUS_COUNT.load(Ordering::Relaxed) > 0
}

pub fn take_needs_continuous_redraw() -> bool {
    NEEDS_CONTINUOUS_REDRAW.swap(false, Ordering::SeqCst)
}

fn request_continuous_redraw() {
    if has_focused_text_input() {
        NEEDS_CONTINUOUS_REDRAW.store(true, Ordering::SeqCst);
    }
}

pub fn request_continuous_redraw_pub() {
    request_continuous_redraw();
}

pub fn take_needs_rebuild() -> bool {
    NEEDS_REBUILD.swap(false, Ordering::SeqCst)
}

pub fn request_rebuild() {
    NEEDS_REBUILD.store(true, Ordering::SeqCst);
}

pub(crate) fn increment_focus_count() {
    let prev = GLOBAL_FOCUS_COUNT.fetch_add(1, Ordering::Relaxed);
    // If this is the first focused text widget, enable continuous redraw for cursor animation
    if prev == 0 {
        notify_continuous_redraw(true);
    }
}

pub(crate) fn decrement_focus_count() {
    let prev = GLOBAL_FOCUS_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
        Some(v.saturating_sub(1))
    });
    // If no more focused text widgets, disable continuous redraw
    if let Ok(prev_val) = prev {
        if prev_val == 1 {
            notify_continuous_redraw(false);
        }
    }
}

pub(crate) fn set_focused_text_input(state: &SharedTextInputData) {
    use blinc_core::events::event_types;

    let mut focused = FOCUSED_TEXT_INPUT.lock().unwrap();

    if let Some(weak) = focused.take() {
        if let Some(prev_state) = weak.upgrade() {
            if !Arc::ptr_eq(&prev_state, state) {
                if let Ok(mut s) = prev_state.lock() {
                    if let Some(new_state) = s.visual.on_event(event_types::BLUR) {
                        s.visual = new_state;
                        decrement_focus_count();
                    }
                }
            }
        }
    }

    blur_focused_text_area();
    *focused = Some(Arc::downgrade(state));
}

pub(crate) fn clear_focused_text_input(state: &SharedTextInputData) {
    let mut focused = FOCUSED_TEXT_INPUT.lock().unwrap();
    if let Some(weak) = focused.as_ref() {
        if let Some(prev_state) = weak.upgrade() {
            if Arc::ptr_eq(&prev_state, state) {
                *focused = None;
            }
        }
    }
}

pub(crate) fn set_focused_text_area(state: &crate::widgets::text_area::SharedTextAreaState) {
    use blinc_core::events::event_types;

    {
        let mut focused = FOCUSED_TEXT_INPUT.lock().unwrap();
        if let Some(weak) = focused.take() {
            if let Some(prev_state) = weak.upgrade() {
                if let Ok(mut s) = prev_state.lock() {
                    if let Some(new_state) = s.visual.on_event(event_types::BLUR) {
                        s.visual = new_state;
                        decrement_focus_count();
                    }
                }
            }
        }
    }

    {
        let mut focused = FOCUSED_TEXT_AREA.lock().unwrap();
        if let Some(weak) = focused.take() {
            if let Some(prev_state) = weak.upgrade() {
                if !Arc::ptr_eq(&prev_state, state) {
                    if let Ok(mut s) = prev_state.lock() {
                        if let Some(new_state) = s.visual.on_event(event_types::BLUR) {
                            s.visual = new_state;
                            decrement_focus_count();
                        }
                    }
                }
            }
        }
        *focused = Some(Arc::downgrade(state));
    }
}

pub(crate) fn clear_focused_text_area(state: &crate::widgets::text_area::SharedTextAreaState) {
    let mut focused = FOCUSED_TEXT_AREA.lock().unwrap();
    if let Some(weak) = focused.as_ref() {
        if let Some(prev_state) = weak.upgrade() {
            if Arc::ptr_eq(&prev_state, state) {
                *focused = None;
            }
        }
    }
}

fn blur_focused_text_area() {
    use blinc_core::events::event_types;

    let mut focused = FOCUSED_TEXT_AREA.lock().unwrap();
    if let Some(weak) = focused.take() {
        if let Some(prev_state) = weak.upgrade() {
            if let Ok(mut s) = prev_state.lock() {
                if let Some(new_state) = s.visual.on_event(event_types::BLUR) {
                    s.visual = new_state;
                    decrement_focus_count();
                }
            }
        }
    }
}

/// Blur all focused text inputs and text areas.
/// Called when clicking outside any text element.
pub fn blur_all_text_inputs() {
    use blinc_core::events::event_types;
    use crate::stateful::refresh_stateful;

    // Blur focused TextInput
    {
        let mut focused = FOCUSED_TEXT_INPUT.lock().unwrap();
        if let Some(weak) = focused.take() {
            if let Some(state) = weak.upgrade() {
                if let Ok(mut s) = state.lock() {
                    if s.visual.is_focused() {
                        if let Some(new_state) = s.visual.on_event(event_types::BLUR) {
                            s.visual = new_state;
                            decrement_focus_count();
                        }
                        // Also update the FSM state to keep in sync
                        let stateful_ref = s.stateful_state.clone();
                        if let Some(ref stateful) = stateful_ref {
                            if let Ok(mut shared) = stateful.lock() {
                                if let Some(new_fsm) = shared.state.on_event(event_types::BLUR) {
                                    shared.state = new_fsm;
                                    shared.needs_visual_update = true;
                                }
                            }
                        }
                        // Trigger visual refresh after releasing the data lock
                        drop(s);
                        if let Some(ref stateful) = stateful_ref {
                            refresh_stateful(stateful);
                        }
                    }
                }
            }
        }
    }

    // Blur focused TextArea
    {
        let mut focused = FOCUSED_TEXT_AREA.lock().unwrap();
        if let Some(weak) = focused.take() {
            if let Some(state) = weak.upgrade() {
                if let Ok(mut s) = state.lock() {
                    if s.visual.is_focused() {
                        if let Some(new_state) = s.visual.on_event(event_types::BLUR) {
                            s.visual = new_state;
                            decrement_focus_count();
                        }
                        // Also update the FSM state to keep in sync
                        let stateful_ref = s.stateful_state.clone();
                        if let Some(ref stateful) = stateful_ref {
                            if let Ok(mut shared) = stateful.lock() {
                                if let Some(new_fsm) = shared.state.on_event(event_types::BLUR) {
                                    shared.state = new_fsm;
                                    shared.needs_visual_update = true;
                                }
                            }
                        }
                        // Trigger visual refresh after releasing the data lock
                        drop(s);
                        if let Some(ref stateful) = stateful_ref {
                            refresh_stateful(stateful);
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Input Types and Validation
// =============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputType {
    #[default]
    Text,
    Number,
    Integer,
    Email,
    Password,
    Url,
    Tel,
    Search,
}

#[derive(Clone, Debug, Default)]
pub struct InputConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub required: bool,
}

impl InputConstraints {
    pub fn max_length(max: usize) -> Self {
        Self {
            max_length: Some(max),
            ..Default::default()
        }
    }

    pub fn required() -> Self {
        Self {
            required: true,
            ..Default::default()
        }
    }

    pub fn number_range(min: f64, max: f64) -> Self {
        Self {
            min_value: Some(min),
            max_value: Some(max),
            ..Default::default()
        }
    }
}

// =============================================================================
// TextInputData - the external state that persists across rebuilds
// =============================================================================

/// Shared text input data handle
pub type SharedTextInputData = Arc<Mutex<TextInputData>>;

/// Text input data (content, cursor, validation)
///
/// This is the EXTERNAL state that persists across rebuilds.
/// Visual state (hover/focus) is managed by the Stateful FSM.
#[derive(Clone)]
pub struct TextInputData {
    pub value: String,
    pub cursor: usize,
    pub selection_start: Option<usize>,
    pub placeholder: String,
    pub input_type: InputType,
    pub constraints: InputConstraints,
    pub disabled: bool,
    pub masked: bool,
    pub is_valid: bool,
    pub visual: TextFieldState,
    pub focus_time_ms: u64,
    pub cursor_state: SharedCursorState,
    /// Horizontal scroll offset for text that exceeds the input width
    pub scroll_offset_x: f32,
    /// Reference to the Stateful's shared state for triggering incremental updates
    pub(crate) stateful_state: Option<SharedState<TextFieldState>>,
}

impl std::fmt::Debug for TextInputData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInputData")
            .field("value", &self.value)
            .field("cursor", &self.cursor)
            .field("selection_start", &self.selection_start)
            .field("placeholder", &self.placeholder)
            .field("input_type", &self.input_type)
            .field("constraints", &self.constraints)
            .field("disabled", &self.disabled)
            .field("masked", &self.masked)
            .field("is_valid", &self.is_valid)
            .field("visual", &self.visual)
            .field("focus_time_ms", &self.focus_time_ms)
            // Skip stateful_state since StatefulInner doesn't implement Debug
            .finish()
    }
}

impl Default for TextInputData {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInputData {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            selection_start: None,
            placeholder: String::new(),
            input_type: InputType::Text,
            constraints: InputConstraints::default(),
            disabled: false,
            masked: false,
            is_valid: true,
            visual: TextFieldState::Idle,
            focus_time_ms: 0,
            cursor_state: cursor_state(),
            scroll_offset_x: 0.0,
            stateful_state: None,
        }
    }

    pub fn with_placeholder(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Self::new()
        }
    }

    pub fn with_value(value: impl Into<String>) -> Self {
        let v: String = value.into();
        let cursor = v.chars().count();
        Self {
            value: v,
            cursor,
            ..Self::new()
        }
    }

    /// Get display text (masked for password, or actual value)
    pub fn display_text(&self) -> String {
        if self.masked {
            "•".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }

    /// Insert text at cursor, respecting input type constraints
    pub fn insert(&mut self, text: &str) {
        // Delete selection first if any
        if let Some(start) = self.selection_start {
            let (from, to) = if start < self.cursor {
                (start, self.cursor)
            } else {
                (self.cursor, start)
            };
            let before: String = self.value.chars().take(from).collect();
            let after: String = self.value.chars().skip(to).collect();
            self.value = before + &after;
            self.cursor = from;
            self.selection_start = None;
        }

        // Filter based on input type
        let filtered: String = match self.input_type {
            InputType::Number => text.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect(),
            InputType::Integer => text.chars().filter(|c| c.is_ascii_digit() || *c == '-').collect(),
            InputType::Tel => text.chars().filter(|c| c.is_ascii_digit() || *c == '+' || *c == '-' || *c == ' ').collect(),
            _ => text.to_string(),
        };

        if filtered.is_empty() {
            return;
        }

        // Check max length
        if let Some(max) = self.constraints.max_length {
            if self.value.chars().count() + filtered.chars().count() > max {
                return;
            }
        }

        // Insert at cursor
        let before: String = self.value.chars().take(self.cursor).collect();
        let after: String = self.value.chars().skip(self.cursor).collect();
        self.value = before + &filtered + &after;
        self.cursor += filtered.chars().count();

        self.validate();
        // NOTE: Don't call trigger_content_refresh() here - caller must do it
        // after releasing the lock to avoid deadlock
    }

    pub fn delete_backward(&mut self) {
        if let Some(start) = self.selection_start {
            let (from, to) = if start < self.cursor { (start, self.cursor) } else { (self.cursor, start) };
            let before: String = self.value.chars().take(from).collect();
            let after: String = self.value.chars().skip(to).collect();
            self.value = before + &after;
            self.cursor = from;
            self.selection_start = None;
        } else if self.cursor > 0 {
            let before: String = self.value.chars().take(self.cursor - 1).collect();
            let after: String = self.value.chars().skip(self.cursor).collect();
            self.value = before + &after;
            self.cursor -= 1;
        }
        self.validate();
        // NOTE: Don't call trigger_content_refresh() here - caller must do it
        // after releasing the lock to avoid deadlock
    }

    pub fn delete_forward(&mut self) {
        if let Some(start) = self.selection_start {
            let (from, to) = if start < self.cursor { (start, self.cursor) } else { (self.cursor, start) };
            let before: String = self.value.chars().take(from).collect();
            let after: String = self.value.chars().skip(to).collect();
            self.value = before + &after;
            self.cursor = from;
            self.selection_start = None;
        } else if self.cursor < self.value.chars().count() {
            let before: String = self.value.chars().take(self.cursor).collect();
            let after: String = self.value.chars().skip(self.cursor + 1).collect();
            self.value = before + &after;
        }
        self.validate();
        // NOTE: Don't call trigger_content_refresh() here - caller must do it
        // after releasing the lock to avoid deadlock
    }

    pub fn move_left(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
        if self.cursor < self.value.chars().count() {
            self.cursor += 1;
        }
    }

    pub fn move_to_start(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
        self.cursor = 0;
    }

    pub fn move_to_end(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
        self.cursor = self.value.chars().count();
    }

    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor = self.value.chars().count();
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selection_start.map(|start| {
            let (from, to) = if start < self.cursor { (start, self.cursor) } else { (self.cursor, start) };
            self.value.chars().skip(from).take(to - from).collect()
        })
    }

    pub fn validate(&mut self) {
        self.is_valid = match self.input_type {
            InputType::Email => self.value.is_empty() || (self.value.contains('@') && self.value.contains('.')),
            InputType::Number => self.value.is_empty() || self.value.parse::<f64>().is_ok(),
            InputType::Integer => self.value.is_empty() || self.value.parse::<i64>().is_ok(),
            InputType::Url => self.value.is_empty() || self.value.starts_with("http://") || self.value.starts_with("https://"),
            _ => true,
        };

        if self.constraints.required && self.value.is_empty() {
            self.is_valid = false;
        }

        if let Some(min) = self.constraints.min_length {
            if self.value.len() < min {
                self.is_valid = false;
            }
        }
    }

    pub fn reset_cursor_blink(&mut self) {
        if let Ok(mut cs) = self.cursor_state.lock() {
            cs.reset_blink();
        }
    }

    pub fn sync_global_selection(&self) {
        if let Some(start) = self.selection_start {
            if start != self.cursor {
                let (from, to) = if start < self.cursor { (start, self.cursor) } else { (self.cursor, start) };
                let selected: String = self.value.chars().skip(from).take(to - from).collect();
                set_selection(selected, SelectionSource::TextInput, true);
            } else {
                clear_selection();
            }
        } else {
            clear_selection();
        }
    }

}

/// Create a shared text input data
pub fn text_input_data() -> SharedTextInputData {
    Arc::new(Mutex::new(TextInputData::new()))
}

/// Create a shared text input data with placeholder
pub fn text_input_data_with_placeholder(placeholder: impl Into<String>) -> SharedTextInputData {
    Arc::new(Mutex::new(TextInputData::with_placeholder(placeholder)))
}

// Backwards compatibility aliases
pub type TextInputState = TextInputData;
pub type SharedTextInputState = SharedTextInputData;

pub fn text_input_state() -> SharedTextInputData {
    text_input_data()
}

pub fn text_input_state_with_placeholder(placeholder: impl Into<String>) -> SharedTextInputData {
    text_input_data_with_placeholder(placeholder)
}

// =============================================================================
// TextInputConfig - visual configuration
// =============================================================================

#[derive(Clone, Debug)]
pub struct TextInputConfig {
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub bg_color: Color,
    pub hover_bg_color: Color,
    pub focused_bg_color: Color,
    pub border_color: Color,
    pub hover_border_color: Color,
    pub focused_border_color: Color,
    pub error_border_color: Color,
    pub cursor_color: Color,
    pub selection_color: Color,
    pub corner_radius: f32,
    pub border_width: f32,
    pub padding_x: f32,
    pub placeholder: String,
}

impl Default for TextInputConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            height: 44.0,
            font_size: 16.0,
            text_color: Color::WHITE,
            placeholder_color: Color::rgba(0.5, 0.5, 0.55, 1.0),
            bg_color: Color::rgba(0.15, 0.15, 0.2, 1.0),
            hover_bg_color: Color::rgba(0.18, 0.18, 0.23, 1.0),
            focused_bg_color: Color::rgba(0.12, 0.12, 0.18, 1.0),
            border_color: Color::rgba(0.3, 0.3, 0.35, 1.0),
            hover_border_color: Color::rgba(0.4, 0.4, 0.45, 1.0),
            focused_border_color: Color::rgba(0.4, 0.6, 1.0, 1.0),
            error_border_color: Color::rgba(1.0, 0.3, 0.3, 1.0),
            cursor_color: Color::rgba(0.4, 0.6, 1.0, 1.0),
            selection_color: Color::rgba(0.3, 0.5, 0.9, 0.4),
            corner_radius: 8.0,
            border_width: 0.5,
            padding_x: 12.0,
            placeholder: String::new(),
        }
    }
}

// =============================================================================
// TextInput Widget
// =============================================================================

/// TextInput widget using FSM-driven Stateful for incremental updates
pub struct TextInput {
    inner: Stateful<TextFieldState>,
    data: SharedTextInputData,
    config: Arc<Mutex<TextInputConfig>>,
    /// Reference to the Stateful's shared state for wiring up to TextInputData
    stateful_state: SharedState<TextFieldState>,
}

impl TextInput {
    /// Create a text input with externally-managed data state
    pub fn new(data: SharedTextInputData) -> Self {
        let config = Arc::new(Mutex::new(TextInputConfig::default()));

        // Get initial visual state and existing stateful_state from data
        let (initial_visual, existing_stateful_state) = {
            let d = data.lock().unwrap();
            (d.visual, d.stateful_state.clone())
        };

        // Reuse existing stateful_state if available, otherwise create new one
        // This ensures state persists across rebuilds (e.g., window resize)
        let stateful_state: SharedState<TextFieldState> = existing_stateful_state
            .unwrap_or_else(|| {
                let new_state = Arc::new(Mutex::new(StatefulInner::new(initial_visual)));
                // Store reference in TextInputData for triggering refreshes
                if let Ok(mut d) = data.lock() {
                    d.stateful_state = Some(Arc::clone(&new_state));
                }
                new_state
            });

        // Create inner Stateful with text input event handlers
        let inner = Self::create_inner_with_handlers(
            Arc::clone(&stateful_state),
            Arc::clone(&data),
        );

        // Register callback immediately so it's available for incremental diff
        // The diff system calls children_builders() before build(), so the callback
        // must be registered here, not in build()
        {
            let config_for_callback = Arc::clone(&config);
            let data_for_callback = Arc::clone(&data);
            let mut shared = stateful_state.lock().unwrap();

            shared.state_callback = Some(Arc::new(move |visual: &TextFieldState, container: &mut Div| {
                let cfg = config_for_callback.lock().unwrap().clone();
                let mut data_guard = data_for_callback.lock().unwrap();

                // Update scroll offset to keep cursor visible
                data_guard.ensure_cursor_visible(&cfg);

                let content = TextInput::build_content(*visual, &data_guard, &cfg);
                container.merge(content);
            }));

            shared.needs_visual_update = true;
        }

        // Ensure state handlers (hover/press) are registered immediately
        // so they're available for incremental diff
        inner.ensure_state_handlers_registered();

        Self {
            inner,
            data,
            config,
            stateful_state,
        }
    }

    /// Create the inner Stateful element with all event handlers registered
    fn create_inner_with_handlers(
        stateful_state: SharedState<TextFieldState>,
        data: SharedTextInputData,
    ) -> Stateful<TextFieldState> {
        use blinc_core::events::event_types;

        let data_for_click = Arc::clone(&data);
        let data_for_text = Arc::clone(&data);
        let data_for_key = Arc::clone(&data);
        let stateful_for_click = Arc::clone(&stateful_state);
        let stateful_for_text = Arc::clone(&stateful_state);
        let stateful_for_key = Arc::clone(&stateful_state);

        Stateful::with_shared_state(stateful_state)
            // Handle mouse down to focus
            .on_mouse_down(move |_ctx| {
                let needs_refresh = {
                    let mut d = match data_for_click.lock() {
                        Ok(d) => d,
                        Err(_) => return,
                    };

                    if d.disabled {
                        return;
                    }

                    // Update FSM state
                    {
                        let mut shared = stateful_for_click.lock().unwrap();
                        if !shared.state.is_focused() {
                            if let Some(new_state) = shared.state.on_event(event_types::POINTER_DOWN)
                                .or_else(|| shared.state.on_event(event_types::FOCUS))
                            {
                                shared.state = new_state;
                                shared.needs_visual_update = true;
                            }
                        }
                    }

                    // Update data state
                    if !d.visual.is_focused() {
                        d.visual = TextFieldState::Focused;
                        d.focus_time_ms = elapsed_ms();
                        d.reset_cursor_blink();
                        increment_focus_count();
                        set_focused_text_input(&data_for_click);
                        request_continuous_redraw();
                    }

                    // Place cursor at end (TODO: proper click-to-position)
                    d.cursor = d.value.chars().count();
                    d.selection_start = None;

                    true // needs refresh
                }; // Lock released here

                // Trigger incremental refresh AFTER releasing the data lock
                if needs_refresh {
                    refresh_stateful(&stateful_for_click);
                }
            })
            // Handle text input
            .on_event(event_types::TEXT_INPUT, move |ctx| {
                let needs_refresh = {
                    let mut d = match data_for_text.lock() {
                        Ok(d) => d,
                        Err(_) => return,
                    };

                    if d.disabled || !d.visual.is_focused() {
                        return;
                    }

                    if let Some(c) = ctx.key_char {
                        d.insert(&c.to_string());
                        d.reset_cursor_blink();
                        tracing::debug!("TextInput received char: {:?}, value: {}", c, d.value);
                        true
                    } else {
                        false
                    }
                }; // Lock released here

                // Trigger incremental refresh AFTER releasing the data lock
                if needs_refresh {
                    refresh_stateful(&stateful_for_text);
                }
            })
            // Handle key down for navigation and deletion
            .on_key_down(move |ctx| {
                let needs_refresh = {
                    let mut d = match data_for_key.lock() {
                        Ok(d) => d,
                        Err(_) => return,
                    };

                    if d.disabled || !d.visual.is_focused() {
                        return;
                    }

                    let mut changed = true;
                    let mut should_blur = false;
                    match ctx.key_code {
                        8 => d.delete_backward(),   // Backspace
                        127 => d.delete_forward(),  // Delete
                        37 => d.move_left(ctx.shift),   // Left arrow
                        39 => d.move_right(ctx.shift),  // Right arrow
                        36 => d.move_to_start(ctx.shift), // Home
                        35 => d.move_to_end(ctx.shift),   // End
                        65 if ctx.meta || ctx.ctrl => d.select_all(), // Ctrl/Cmd+A
                        27 => {
                            // Escape - blur the input
                            should_blur = true;
                            changed = true;
                        }
                        _ => changed = false,
                    }

                    if changed && !should_blur {
                        d.reset_cursor_blink();
                        d.sync_global_selection();
                    }

                    (changed, should_blur)
                }; // Lock released here

                // Handle blur (Escape key)
                if needs_refresh.1 {
                    blur_all_text_inputs();
                } else if needs_refresh.0 {
                    // Trigger incremental refresh AFTER releasing the data lock
                    refresh_stateful(&stateful_for_key);
                }
            })
    }

    /// Build the content div based on current visual state and data
    fn build_content(visual: TextFieldState, data: &TextInputData, config: &TextInputConfig) -> Div {
        let display = if data.value.is_empty() {
            if !data.placeholder.is_empty() {
                data.placeholder.clone()
            } else {
                config.placeholder.clone()
            }
        } else {
            data.display_text()
        };

        let text_color = if data.value.is_empty() {
            config.placeholder_color
        } else if data.disabled {
            Color::rgba(0.4, 0.4, 0.4, 1.0)
        } else {
            config.text_color
        };

        // Visual state-based styling
        let (bg, border) = match visual {
            TextFieldState::Idle => (config.bg_color, config.border_color),
            TextFieldState::Hovered => (config.hover_bg_color, config.hover_border_color),
            TextFieldState::Focused | TextFieldState::FocusedHovered => {
                (config.focused_bg_color, config.focused_border_color)
            }
            TextFieldState::Disabled => (
                Color::rgba(0.12, 0.12, 0.15, 0.5),
                Color::rgba(0.25, 0.25, 0.3, 0.5),
            ),
        };

        let border = if !data.is_valid && !data.value.is_empty() {
            config.error_border_color
        } else {
            border
        };

        let is_focused = visual.is_focused();
        let cursor_color = config.cursor_color;
        let selection_color = config.selection_color;
        let cursor_pos = data.cursor;
        let cursor_height = config.font_size * 1.2;

        let selection_range: Option<(usize, usize)> = data.selection_start.map(|start| {
            if start < cursor_pos { (start, cursor_pos) } else { (cursor_pos, start) }
        });

        let cursor_state_for_canvas = Arc::clone(&data.cursor_state);

        let cursor_x = if cursor_pos > 0 && !display.is_empty() {
            let text_before: String = display.chars().take(cursor_pos).collect();
            crate::text_measure::measure_text(&text_before, config.font_size).width
        } else {
            0.0
        };

        // Build inner content
        let mut inner_content = div()
            .w_full()
            .h_full()
            .bg(bg)
            .rounded(config.corner_radius)
            .padding_x_px(config.padding_x)
            .relative()
            .flex_row()
            .justify_start()
            .items_center()
            .overflow_clip();

        // Add text content
        if !display.is_empty() {
            if let Some((sel_start, sel_end)) = selection_range {
                let mut text_container = div().flex_row().items_center();

                let before_sel: String = display.chars().take(sel_start).collect();
                if !before_sel.is_empty() {
                    text_container = text_container.child(
                        text(&before_sel).size(config.font_size).color(text_color).text_left().no_wrap().v_center(),
                    );
                }

                let selected: String = display.chars().skip(sel_start).take(sel_end - sel_start).collect();
                if !selected.is_empty() {
                    text_container = text_container.child(
                        div().bg(selection_color).rounded(2.0).child(
                            text(&selected).size(config.font_size).color(text_color).text_left().no_wrap().v_center(),
                        ),
                    );
                }

                let after_sel: String = display.chars().skip(sel_end).collect();
                if !after_sel.is_empty() {
                    text_container = text_container.child(
                        text(&after_sel).size(config.font_size).color(text_color).text_left().no_wrap().v_center(),
                    );
                }

                inner_content = inner_content.child(text_container);
            } else {
                inner_content = inner_content.child(
                    text(&display).size(config.font_size).color(text_color).text_left().no_wrap().v_center(),
                );
            }
        }

        // Add cursor via canvas (AnimatedValue-driven, no rebuilds for blink)
        if is_focused && selection_range.is_none() {
            let cursor_left = config.padding_x + cursor_x;
            let cursor_top = (config.height - config.border_width * 2.0 - cursor_height) / 2.0;

            {
                if let Ok(mut cs) = cursor_state_for_canvas.lock() {
                    cs.visible = true;
                    cs.color = cursor_color;
                    cs.x = cursor_x;
                    cs.animation = CursorAnimation::SmoothFade;
                }
            }

            let cursor_state_clone = Arc::clone(&cursor_state_for_canvas);
            let cursor_canvas = canvas(
                move |ctx: &mut dyn blinc_core::DrawContext, bounds: crate::canvas::CanvasBounds| {
                    let cs = cursor_state_clone.lock().unwrap();
                    if !cs.visible { return; }

                    let opacity = cs.current_opacity();
                    if opacity < 0.01 { return; }

                    let color = blinc_core::Color::rgba(cs.color.r, cs.color.g, cs.color.b, cs.color.a * opacity);
                    ctx.fill_rect(
                        blinc_core::Rect::new(0.0, 0.0, cs.width, bounds.height),
                        blinc_core::CornerRadius::default(),
                        blinc_core::Brush::Solid(color),
                    );
                },
            )
            .absolute()
            .left(cursor_left)
            .top(cursor_top)
            .w(2.0)
            .h(cursor_height);

            inner_content = inner_content.child(cursor_canvas);
        } else {
            if let Ok(mut cs) = cursor_state_for_canvas.lock() {
                cs.visible = false;
            }
        }

        // Outer container (border)
        div()
            .w_full()
            .h_full()
            .bg(border)
            .rounded(config.corner_radius)
            .p(config.border_width)
            .child(inner_content)
    }

    // Builder methods that forward to inner Stateful
    pub fn w(mut self, px: f32) -> Self {
        {
            let mut cfg = self.config.lock().unwrap();
            cfg.width = px;
        }
        self.inner = std::mem::take(&mut self.inner).w(px);
        self
    }

    pub fn h(mut self, px: f32) -> Self {
        {
            let mut cfg = self.config.lock().unwrap();
            cfg.height = px;
        }
        self.inner = std::mem::take(&mut self.inner).h(px);
        self
    }

    pub fn placeholder(self, text: impl Into<String>) -> Self {
        let placeholder = text.into();
        self.config.lock().unwrap().placeholder = placeholder.clone();
        if let Ok(mut d) = self.data.lock() {
            d.placeholder = placeholder;
        }
        self
    }

    pub fn input_type(self, input_type: InputType) -> Self {
        if let Ok(mut d) = self.data.lock() {
            d.input_type = input_type;
        }
        self
    }

    pub fn disabled(self, disabled: bool) -> Self {
        if let Ok(mut d) = self.data.lock() {
            d.disabled = disabled;
            if disabled {
                d.visual = TextFieldState::Disabled;
            }
        }
        self
    }

    pub fn masked(self, masked: bool) -> Self {
        if let Ok(mut d) = self.data.lock() {
            d.masked = masked;
        }
        self
    }

    pub fn max_length(self, max: usize) -> Self {
        if let Ok(mut d) = self.data.lock() {
            d.constraints.max_length = Some(max);
        }
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.config.lock().unwrap().corner_radius = radius;
        self.inner = std::mem::take(&mut self.inner).rounded(radius);
        self
    }

    pub fn shadow_sm(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).shadow_sm();
        self
    }

    pub fn shadow_md(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).shadow_md();
        self
    }

    pub fn flex_grow(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).flex_grow();
        self
    }
}

/// Create a text input widget
pub fn text_input(data: &SharedTextInputData) -> TextInput {
    let config = TextInputConfig::default();
    TextInput::new(Arc::clone(data)).w(config.width).h(config.height)
}

impl ElementBuilder for TextInput {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        // Set base render props for incremental updates
        // Note: callback and handlers are registered in new() so they're available for incremental diff
        {
            let mut shared = self.stateful_state.lock().unwrap();
            shared.base_render_props = Some(self.inner.inner_render_props());
        }

        self.inner.build(tree)
    }

    fn render_props(&self) -> RenderProps {
        self.inner.render_props()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn element_type_id(&self) -> crate::div::ElementTypeId {
        crate::div::ElementTypeId::Div
    }

    fn event_handlers(&self) -> Option<&crate::event_handler::EventHandlers> {
        self.inner.event_handlers()
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        self.inner.layout_style()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_data_insert() {
        let mut data = TextInputData::new();
        data.stateful_state = None; // No refresh in tests

        data.insert("hello");
        assert_eq!(data.value, "hello");
        assert_eq!(data.cursor, 5);

        data.cursor = 0;
        data.insert("world ");
        assert_eq!(data.value, "world hello");
    }

    #[test]
    fn test_text_input_data_delete() {
        let mut data = TextInputData::with_value("hello");
        data.stateful_state = None;

        data.cursor = 5;
        data.delete_backward();
        assert_eq!(data.value, "hell");

        data.cursor = 0;
        data.delete_forward();
        assert_eq!(data.value, "ell");
    }

    #[test]
    fn test_input_type_filtering() {
        let mut data = TextInputData::new();
        data.stateful_state = None;
        data.input_type = InputType::Number;

        data.insert("123.45");
        assert_eq!(data.value, "123.45");

        data.value.clear();
        data.cursor = 0;
        data.insert("abc123");
        assert_eq!(data.value, "123");
    }
}
