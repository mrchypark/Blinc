//! Ready-to-use TextArea widget
//!
//! Multi-line text area with:
//! - Multi-line text editing
//! - Row/column sizing (like HTML textarea)
//! - Cursor and selection
//! - Visual states: idle, hovered, focused
//! - Built-in styling that just works
//! - Inherits ALL Div methods for full layout control via Deref

use std::sync::{Arc, Mutex};

use blinc_core::Color;

use crate::canvas::canvas;
use crate::div::{div, Div, ElementBuilder};
use crate::element::RenderProps;
use crate::stateful::{refresh_stateful, SharedState, StatefulInner, StateTransitions, Stateful, TextFieldState};
use crate::text::text;
use crate::tree::{LayoutNodeId, LayoutTree};
use crate::widgets::cursor::{cursor_state, CursorAnimation, SharedCursorState};
use crate::widgets::scroll::{scroll, Scroll, ScrollDirection, ScrollPhysics, SharedScrollPhysics};
use crate::widgets::text_input::{
    decrement_focus_count, elapsed_ms, increment_focus_count,
    request_continuous_redraw_pub, set_focused_text_area,
};

/// Position in a multi-line text (line and column)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TextPosition {
    /// Line index (0-based)
    pub line: usize,
    /// Column index (character offset within line, 0-based)
    pub column: usize,
}

impl TextPosition {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// TextArea configuration
#[derive(Clone)]
pub struct TextAreaConfig {
    /// Placeholder text shown when empty
    pub placeholder: String,
    /// Width of the text area (can be overridden by cols)
    pub width: f32,
    /// Height of the text area (can be overridden by rows)
    pub height: f32,
    /// Number of visible rows (overrides height if set)
    pub rows: Option<usize>,
    /// Number of visible columns/character width (overrides width if set)
    pub cols: Option<usize>,
    /// Font size
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Approximate character width in ems (for cols calculation)
    pub char_width_ratio: f32,
    /// Text color
    pub text_color: Color,
    /// Placeholder text color
    pub placeholder_color: Color,
    /// Background color
    pub bg_color: Color,
    /// Focused background color
    pub focused_bg_color: Color,
    /// Border color
    pub border_color: Color,
    /// Focused border color
    pub focused_border_color: Color,
    /// Border width
    pub border_width: f32,
    /// Corner radius
    pub corner_radius: f32,
    /// Horizontal padding
    pub padding_x: f32,
    /// Vertical padding
    pub padding_y: f32,
    /// Cursor color
    pub cursor_color: Color,
    /// Selection color
    pub selection_color: Color,
    /// Whether the text area is disabled
    pub disabled: bool,
    /// Maximum character count (0 = unlimited)
    pub max_length: usize,
}

impl Default for TextAreaConfig {
    fn default() -> Self {
        Self {
            placeholder: String::new(),
            width: 300.0,
            height: 120.0,
            rows: None,
            cols: None,
            font_size: 14.0,
            line_height: 1.4,
            char_width_ratio: 0.6,
            text_color: Color::rgba(0.9, 0.9, 0.9, 1.0),
            placeholder_color: Color::rgba(0.5, 0.5, 0.5, 1.0),
            bg_color: Color::rgba(0.15, 0.15, 0.2, 1.0),
            focused_bg_color: Color::rgba(0.18, 0.18, 0.25, 1.0),
            border_color: Color::rgba(0.3, 0.3, 0.35, 1.0),
            focused_border_color: Color::rgba(0.4, 0.6, 1.0, 1.0),
            border_width: 1.0,
            corner_radius: 8.0,
            padding_x: 12.0,
            padding_y: 10.0,
            cursor_color: Color::rgba(0.4, 0.6, 1.0, 1.0),
            selection_color: Color::rgba(0.4, 0.6, 1.0, 0.3),
            disabled: false,
            max_length: 0,
        }
    }
}

impl TextAreaConfig {
    /// Calculate the effective width based on cols or explicit width
    pub fn effective_width(&self) -> f32 {
        if let Some(cols) = self.cols {
            let char_width = self.font_size * self.char_width_ratio;
            cols as f32 * char_width + self.padding_x * 2.0 + self.border_width * 2.0
        } else {
            self.width
        }
    }

    /// Calculate the effective height based on rows or explicit height
    pub fn effective_height(&self) -> f32 {
        if let Some(rows) = self.rows {
            let single_line_height = self.font_size * self.line_height;
            rows as f32 * single_line_height + self.padding_y * 2.0 + self.border_width * 2.0
        } else {
            self.height
        }
    }
}

/// TextArea widget state
#[derive(Clone)]
pub struct TextAreaState {
    /// Lines of text
    pub lines: Vec<String>,
    /// Cursor position
    pub cursor: TextPosition,
    /// Selection start position (if selecting)
    pub selection_start: Option<TextPosition>,
    /// Visual state for styling
    pub visual: TextFieldState,
    /// Placeholder text
    pub placeholder: String,
    /// Whether disabled
    pub disabled: bool,
    /// Time when focus was gained (for cursor blinking)
    /// Stored as milliseconds since some epoch (e.g., app start)
    pub focus_time_ms: u64,
    /// Cursor blink interval in milliseconds
    pub cursor_blink_interval_ms: u64,
    /// Canvas-based cursor state for smooth animation
    pub cursor_state: SharedCursorState,
    /// Shared scroll physics for vertical scrolling
    pub(crate) scroll_physics: SharedScrollPhysics,
    /// Cached viewport height for scroll calculations
    pub(crate) viewport_height: f32,
    /// Cached line height for scroll calculations
    pub(crate) line_height: f32,
    /// Reference to the Stateful's shared state for triggering incremental updates
    pub(crate) stateful_state: Option<SharedState<TextFieldState>>,
}

impl std::fmt::Debug for TextAreaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextAreaState")
            .field("lines", &self.lines)
            .field("cursor", &self.cursor)
            .field("selection_start", &self.selection_start)
            .field("visual", &self.visual)
            .field("placeholder", &self.placeholder)
            .field("disabled", &self.disabled)
            .field("focus_time_ms", &self.focus_time_ms)
            .field("cursor_blink_interval_ms", &self.cursor_blink_interval_ms)
            // Skip stateful_state since StatefulInner doesn't implement Debug
            .finish()
    }
}

impl Default for TextAreaState {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: TextPosition::default(),
            selection_start: None,
            visual: TextFieldState::Idle,
            placeholder: String::new(),
            disabled: false,
            focus_time_ms: 0,
            cursor_blink_interval_ms: 530, // Standard cursor blink rate (~530ms)
            cursor_state: cursor_state(),
            scroll_physics: Arc::new(Mutex::new(ScrollPhysics::default())),
            viewport_height: 120.0, // Default height from TextAreaConfig
            line_height: 14.0 * 1.4, // Default font_size * line_height
            stateful_state: None,
        }
    }
}

impl TextAreaState {
    /// Create new text area state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with initial value
    pub fn with_value(value: impl Into<String>) -> Self {
        let value = value.into();
        let lines: Vec<String> = if value.is_empty() {
            vec![String::new()]
        } else {
            value.lines().map(|s| s.to_string()).collect()
        };
        let cursor = TextPosition::new(
            lines.len().saturating_sub(1),
            lines.last().map(|l| l.chars().count()).unwrap_or(0),
        );
        Self {
            lines,
            cursor,
            ..Default::default()
        }
    }

    /// Create with placeholder
    pub fn with_placeholder(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Default::default()
        }
    }

    /// Get the full text value
    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    /// Set the text value
    pub fn set_value(&mut self, value: &str) {
        self.lines = if value.is_empty() {
            vec![String::new()]
        } else {
            value.lines().map(|s| s.to_string()).collect()
        };
        self.cursor = TextPosition::new(
            self.lines.len().saturating_sub(1),
            self.lines.last().map(|l| l.chars().count()).unwrap_or(0),
        );
        self.selection_start = None;
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get a specific line
    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    /// Is empty?
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Is focused?
    pub fn is_focused(&self) -> bool {
        self.visual.is_focused()
    }

    /// Check if cursor should be visible based on current time
    /// Returns true if cursor is in the "on" phase of blinking
    pub fn is_cursor_visible(&self, current_time_ms: u64) -> bool {
        if self.cursor_blink_interval_ms == 0 {
            return true; // No blinking, always visible
        }
        let elapsed = current_time_ms.saturating_sub(self.focus_time_ms);
        let phase = (elapsed / self.cursor_blink_interval_ms) % 2;
        phase == 0
    }

    /// Reset cursor blink (call when focus gained or cursor moved)
    pub fn reset_cursor_blink(&mut self) {
        self.focus_time_ms = elapsed_ms();
        // Also reset the canvas cursor state for smooth animation
        if let Ok(mut cs) = self.cursor_state.lock() {
            cs.reset_blink();
        }
    }

    /// Insert text at cursor
    pub fn insert(&mut self, text: &str) {
        self.delete_selection();

        if text.contains('\n') {
            for (i, part) in text.split('\n').enumerate() {
                if i > 0 {
                    self.insert_newline();
                }
                self.insert_text(part);
            }
        } else {
            self.insert_text(text);
        }
    }

    fn insert_text(&mut self, text: &str) {
        let line_idx = self.cursor.line.min(self.lines.len().saturating_sub(1));
        let byte_pos = char_to_byte_pos(&self.lines[line_idx], self.cursor.column);
        self.lines[line_idx].insert_str(byte_pos, text);
        self.cursor.column += text.chars().count();
    }

    /// Insert a newline at cursor
    pub fn insert_newline(&mut self) {
        self.delete_selection();

        let line_idx = self.cursor.line.min(self.lines.len().saturating_sub(1));
        let byte_pos = char_to_byte_pos(&self.lines[line_idx], self.cursor.column);

        let after = self.lines[line_idx].split_off(byte_pos);
        self.lines.insert(line_idx + 1, after);

        self.cursor.line += 1;
        self.cursor.column = 0;
    }

    /// Delete character before cursor (backspace)
    pub fn delete_backward(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor.column > 0 {
            let start_byte =
                char_to_byte_pos(&self.lines[self.cursor.line], self.cursor.column - 1);
            let end_byte = char_to_byte_pos(&self.lines[self.cursor.line], self.cursor.column);
            self.lines[self.cursor.line].replace_range(start_byte..end_byte, "");
            self.cursor.column -= 1;
        } else if self.cursor.line > 0 {
            let current_line = self.lines.remove(self.cursor.line);
            self.cursor.line -= 1;
            self.cursor.column = self.lines[self.cursor.line].chars().count();
            self.lines[self.cursor.line].push_str(&current_line);
        }
    }

    /// Delete character after cursor (delete)
    pub fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }

        let line_len = self.lines[self.cursor.line].chars().count();
        if self.cursor.column < line_len {
            let start_byte = char_to_byte_pos(&self.lines[self.cursor.line], self.cursor.column);
            let end_byte = char_to_byte_pos(&self.lines[self.cursor.line], self.cursor.column + 1);
            self.lines[self.cursor.line].replace_range(start_byte..end_byte, "");
        } else if self.cursor.line < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor.line + 1);
            self.lines[self.cursor.line].push_str(&next_line);
        }
    }

    /// Delete selected text
    fn delete_selection(&mut self) -> bool {
        if let Some(start) = self.selection_start {
            let (from, to) = self.order_positions(start, self.cursor);

            if from != to {
                if from.line == to.line {
                    let start_byte = char_to_byte_pos(&self.lines[from.line], from.column);
                    let end_byte = char_to_byte_pos(&self.lines[from.line], to.column);
                    self.lines[from.line].replace_range(start_byte..end_byte, "");
                } else {
                    let from_byte = char_to_byte_pos(&self.lines[from.line], from.column);
                    self.lines[from.line].truncate(from_byte);

                    let to_byte = char_to_byte_pos(&self.lines[to.line], to.column);
                    let after_text = self.lines[to.line][to_byte..].to_string();
                    self.lines[from.line].push_str(&after_text);

                    for _ in from.line + 1..=to.line {
                        if from.line + 1 < self.lines.len() {
                            self.lines.remove(from.line + 1);
                        }
                    }
                }

                self.cursor = from;
                self.selection_start = None;
                return true;
            }
        }
        self.selection_start = None;
        false
    }

    /// Order two positions (returns (earlier, later))
    fn order_positions(&self, a: TextPosition, b: TextPosition) -> (TextPosition, TextPosition) {
        if a.line < b.line || (a.line == b.line && a.column <= b.column) {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            if let Some(start) = self.selection_start {
                let (from, _) = self.order_positions(start, self.cursor);
                self.cursor = from;
                self.selection_start = None;
                return;
            }
        }

        if self.cursor.column > 0 {
            self.cursor.column -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.column = self.lines[self.cursor.line].chars().count();
        }

        if !select {
            self.selection_start = None;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            if let Some(start) = self.selection_start {
                let (_, to) = self.order_positions(start, self.cursor);
                self.cursor = to;
                self.selection_start = None;
                return;
            }
        }

        let line_len = self.lines[self.cursor.line].chars().count();
        if self.cursor.column < line_len {
            self.cursor.column += 1;
        } else if self.cursor.line < self.lines.len() - 1 {
            self.cursor.line += 1;
            self.cursor.column = 0;
        }

        if !select {
            self.selection_start = None;
        }
    }

    /// Move cursor up
    pub fn move_up(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            self.selection_start = None;
        }

        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            let line_len = self.lines[self.cursor.line].chars().count();
            self.cursor.column = self.cursor.column.min(line_len);
        }
    }

    /// Move cursor down
    pub fn move_down(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            self.selection_start = None;
        }

        if self.cursor.line < self.lines.len() - 1 {
            self.cursor.line += 1;
            let line_len = self.lines[self.cursor.line].chars().count();
            self.cursor.column = self.cursor.column.min(line_len);
        }
    }

    /// Move to start of line
    pub fn move_to_line_start(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            self.selection_start = None;
        }
        self.cursor.column = 0;
    }

    /// Move to end of line
    pub fn move_to_line_end(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            self.selection_start = None;
        }
        self.cursor.column = self.lines[self.cursor.line].chars().count();
    }

    /// Move to start of text
    pub fn move_to_start(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            self.selection_start = None;
        }
        self.cursor = TextPosition::new(0, 0);
    }

    /// Move to end of text
    pub fn move_to_end(&mut self, select: bool) {
        if select && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        } else if !select {
            self.selection_start = None;
        }
        let last_line = self.lines.len().saturating_sub(1);
        self.cursor = TextPosition::new(last_line, self.lines[last_line].chars().count());
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection_start = Some(TextPosition::new(0, 0));
        let last_line = self.lines.len().saturating_sub(1);
        self.cursor = TextPosition::new(last_line, self.lines[last_line].chars().count());
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.selection_start.map(|start| {
            let (from, to) = self.order_positions(start, self.cursor);

            if from.line == to.line {
                self.lines[from.line]
                    .chars()
                    .skip(from.column)
                    .take(to.column - from.column)
                    .collect()
            } else {
                let mut result = String::new();
                result.extend(self.lines[from.line].chars().skip(from.column));

                for line in &self.lines[from.line + 1..to.line] {
                    result.push('\n');
                    result.push_str(line);
                }

                if to.line > from.line {
                    result.push('\n');
                    result.extend(self.lines[to.line].chars().take(to.column));
                }

                result
            }
        })
    }

    /// Ensure the cursor is visible by adjusting scroll offset if needed
    ///
    /// This should be called after any cursor movement to auto-scroll
    /// when the cursor moves outside the visible area.
    pub fn ensure_cursor_visible(&mut self, line_height: f32, viewport_height: f32) {
        let cursor_y = self.cursor.line as f32 * line_height;
        let cursor_bottom = cursor_y + line_height;

        // Get current scroll offset from physics (offset_y is negative when scrolled down)
        let mut physics = self.scroll_physics.lock().unwrap();
        let current_offset = -physics.offset_y; // Convert to positive scroll offset

        // If cursor is above visible area, scroll up
        let mut new_offset = current_offset;
        if cursor_y < current_offset {
            new_offset = cursor_y;
        }

        // If cursor is below visible area, scroll down
        if cursor_bottom > current_offset + viewport_height {
            new_offset = cursor_bottom - viewport_height;
        }

        // Clamp scroll offset to valid range
        let content_height = self.lines.len() as f32 * line_height;
        let max_scroll = (content_height - viewport_height).max(0.0);
        new_offset = new_offset.clamp(0.0, max_scroll);

        // Update physics offset (negative for scroll physics convention)
        physics.offset_y = -new_offset;
    }

    /// Get current scroll offset (positive value, 0 = top)
    pub fn scroll_offset(&self) -> f32 {
        -self.scroll_physics.lock().unwrap().offset_y
    }
}

/// Convert character index to byte index
fn char_to_byte_pos(line: &str, char_pos: usize) -> usize {
    line.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or(line.len())
}

/// Shared text area state handle
pub type SharedTextAreaState = Arc<Mutex<TextAreaState>>;

/// Create a shared text area state
pub fn text_area_state() -> SharedTextAreaState {
    Arc::new(Mutex::new(TextAreaState::new()))
}

/// Create a shared text area state with placeholder
pub fn text_area_state_with_placeholder(placeholder: impl Into<String>) -> SharedTextAreaState {
    Arc::new(Mutex::new(TextAreaState::with_placeholder(placeholder)))
}

/// Ready-to-use text area element
///
/// Uses FSM-driven state management via `Stateful<TextFieldState>` for visual states
/// while maintaining separate text content state for editing.
///
/// Usage: `text_area(&state).rows(4).w(400.0).rounded(12.0)`
pub struct TextArea {
    /// Inner Stateful element for FSM-driven visual states
    inner: Stateful<TextFieldState>,
    /// Text area state (content, cursor, etc.)
    state: SharedTextAreaState,
    /// Text area configuration
    config: Arc<Mutex<TextAreaConfig>>,
}

impl TextArea {
    /// Create a new text area with shared state
    pub fn new(state: &SharedTextAreaState) -> Self {
        let config = Arc::new(Mutex::new(TextAreaConfig::default()));
        let cfg = config.lock().unwrap();
        let default_width = cfg.effective_width();
        let default_height = cfg.effective_height();
        drop(cfg);

        // Get initial visual state and existing stateful_state from data
        let (initial_visual, existing_stateful_state) = {
            let d = state.lock().unwrap();
            (d.visual, d.stateful_state.clone())
        };

        // Reuse existing stateful_state if available, otherwise create new one
        // This ensures state persists across rebuilds (e.g., window resize)
        let shared_state: SharedState<TextFieldState> = existing_stateful_state
            .unwrap_or_else(|| {
                let new_state = Arc::new(Mutex::new(StatefulInner::new(initial_visual)));
                // Store reference in TextAreaState for triggering refreshes
                if let Ok(mut d) = state.lock() {
                    d.stateful_state = Some(Arc::clone(&new_state));
                }
                new_state
            });

        // Create inner Stateful with text area event handlers
        let inner = Self::create_inner_with_handlers(
            Arc::clone(&shared_state),
            Arc::clone(state),
        ).w(default_width).h(default_height);

        // Register callback immediately so it's available for incremental diff
        // The diff system calls children_builders() before build(), so the callback
        // must be registered here, not in build()
        {
            let config_for_callback = Arc::clone(&config);
            let data_for_callback = Arc::clone(state);
            let mut shared = shared_state.lock().unwrap();

            shared.state_callback = Some(Arc::new(move |visual: &TextFieldState, container: &mut Div| {
                let cfg = config_for_callback.lock().unwrap().clone();
                let mut data_guard = data_for_callback.lock().unwrap();

                // Sync visual state to data so it matches the FSM
                data_guard.visual = *visual;

                // Update cached scroll dimensions from config
                let line_height = cfg.font_size * cfg.line_height;
                let viewport_height = cfg.effective_height() - cfg.padding_y * 2.0 - cfg.border_width * 2.0;
                data_guard.line_height = line_height;
                data_guard.viewport_height = viewport_height;

                let content = TextArea::build_content(*visual, &data_guard, &cfg);
                container.merge(content);
            }));

            shared.needs_visual_update = true;
        }

        // Ensure state handlers (hover/press) are registered immediately
        // so they're available for incremental diff
        inner.ensure_state_handlers_registered();

        let textarea = Self {
            inner,
            state: Arc::clone(state),
            config,
        };

        // Initialize scroll dimensions from default config
        textarea.update_scroll_dimensions();

        textarea
    }

    /// Create the inner Stateful element with all event handlers registered
    fn create_inner_with_handlers(
        shared_state: SharedState<TextFieldState>,
        data: SharedTextAreaState,
    ) -> Stateful<TextFieldState> {
        use blinc_core::events::event_types;

        let data_for_click = Arc::clone(&data);
        let data_for_text = Arc::clone(&data);
        let data_for_key = Arc::clone(&data);
        let shared_for_click = Arc::clone(&shared_state);
        let shared_for_text = Arc::clone(&shared_state);
        let shared_for_key = Arc::clone(&shared_state);

        Stateful::with_shared_state(shared_state)
            // Handle mouse down to focus
            .on_mouse_down(move |_ctx| {
                // First, forcibly blur any previously focused text input/area
                set_focused_text_area(&data_for_click);

                let needs_refresh = {
                    let mut d = match data_for_click.lock() {
                        Ok(d) => d,
                        Err(_) => return,
                    };

                    if d.disabled {
                        return;
                    }

                    // Set focus via FSM transition
                    {
                        let mut shared = shared_for_click.lock().unwrap();
                        if !shared.state.is_focused() {
                            // Transition to focused state
                            if let Some(new_state) = shared.state.on_event(event_types::POINTER_DOWN)
                                .or_else(|| shared.state.on_event(event_types::FOCUS))
                            {
                                shared.state = new_state;
                                shared.needs_visual_update = true;
                            }
                        }
                    }

                    // Update data state
                    let was_focused = d.visual.is_focused();
                    if !was_focused {
                        d.visual = TextFieldState::Focused;
                        d.reset_cursor_blink();
                        increment_focus_count();
                        request_continuous_redraw_pub();
                    }

                    true // needs refresh
                }; // Lock released here

                // Trigger incremental refresh AFTER releasing the data lock
                if needs_refresh {
                    refresh_stateful(&shared_for_click);
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
                        // Ensure cursor is visible after text insertion (use cached values)
                        let line_height = d.line_height;
                        let viewport_height = d.viewport_height;
                        d.ensure_cursor_visible(line_height, viewport_height);
                        tracing::debug!(
                            "TextArea received char: {:?}, value: {}",
                            c,
                            d.value()
                        );
                        true
                    } else {
                        false
                    }
                }; // Lock released here

                // Trigger incremental refresh AFTER releasing the data lock
                if needs_refresh {
                    refresh_stateful(&shared_for_text);
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

                    let mut cursor_changed = true;
                    let mut should_blur = false;
                    match ctx.key_code {
                        8 => {
                            // Backspace
                            d.delete_backward();
                            tracing::debug!("TextArea backspace, value: {}", d.value());
                        }
                        127 => {
                            // Delete
                            d.delete_forward();
                        }
                        13 => {
                            // Enter - insert newline
                            d.insert_newline();
                            tracing::debug!("TextArea newline, lines: {}", d.line_count());
                        }
                        37 => {
                            // Left arrow
                            d.move_left(ctx.shift);
                        }
                        39 => {
                            // Right arrow
                            d.move_right(ctx.shift);
                        }
                        38 => {
                            // Up arrow
                            d.move_up(ctx.shift);
                        }
                        40 => {
                            // Down arrow
                            d.move_down(ctx.shift);
                        }
                        36 => {
                            // Home
                            d.move_to_line_start(ctx.shift);
                        }
                        35 => {
                            // End
                            d.move_to_line_end(ctx.shift);
                        }
                        27 => {
                            // Escape - blur the textarea
                            should_blur = true;
                            cursor_changed = true;
                        }
                        _ => {
                            cursor_changed = false;
                        }
                    }

                    if cursor_changed && !should_blur {
                        d.reset_cursor_blink();
                        // Ensure cursor is visible (auto-scroll if needed, use cached values)
                        let line_height = d.line_height;
                        let viewport_height = d.viewport_height;
                        d.ensure_cursor_visible(line_height, viewport_height);
                    }

                    (cursor_changed, should_blur)
                }; // Lock released here

                // Handle blur (Escape key)
                if needs_refresh.1 {
                    crate::widgets::text_input::blur_all_text_inputs();
                } else if needs_refresh.0 {
                    // Trigger incremental refresh AFTER releasing the data lock
                    refresh_stateful(&shared_for_key);
                }
            })
            // Note: Scroll events are handled by the scroll() widget inside build_content
    }

    /// Build the content div based on current visual state and data
    fn build_content(
        visual: TextFieldState,
        data: &TextAreaState,
        config: &TextAreaConfig,
    ) -> Div {
        // Visual state-based styling
        let (bg, border) = match visual {
            TextFieldState::Idle => (config.bg_color, config.border_color),
            TextFieldState::Hovered => (
                Color::rgba(0.18, 0.18, 0.23, 1.0),
                Color::rgba(0.4, 0.4, 0.45, 1.0),
            ),
            TextFieldState::Focused | TextFieldState::FocusedHovered => {
                (config.focused_bg_color, config.focused_border_color)
            }
            TextFieldState::Disabled => (
                Color::rgba(0.12, 0.12, 0.15, 0.5),
                Color::rgba(0.25, 0.25, 0.3, 0.5),
            ),
        };

        let text_color = if data.is_empty() {
            config.placeholder_color
        } else if data.disabled {
            Color::rgba(0.4, 0.4, 0.4, 1.0)
        } else {
            config.text_color
        };

        // Check if cursor should be shown (focused state)
        let is_focused = visual.is_focused();
        let cursor_color = config.cursor_color;

        // Get cursor position
        let cursor_line = data.cursor.line;
        let cursor_col = data.cursor.column;

        // Cursor dimensions
        let cursor_height = config.font_size * 1.2;
        let line_height = config.font_size * config.line_height;

        // Calculate cursor x position using text measurement
        let cursor_x = if cursor_col > 0 && cursor_line < data.lines.len() {
            let line_text = &data.lines[cursor_line];
            let text_before: String = line_text.chars().take(cursor_col).collect();
            crate::text_measure::measure_text(&text_before, config.font_size).width
        } else {
            0.0
        };

        // Clone the cursor state for the canvas callback
        let cursor_state_for_canvas = Arc::clone(&data.cursor_state);

        // Build cursor canvas element (if focused)
        // The cursor is positioned inside the scroll content so it scrolls with text
        let cursor_canvas_opt = if is_focused {
            // Cursor top is based on line position plus vertical centering within line
            let cursor_top = (cursor_line as f32 * line_height)
                + (line_height - cursor_height) / 2.0;
            let cursor_left = cursor_x;

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
                move |ctx: &mut dyn blinc_core::DrawContext,
                      bounds: crate::canvas::CanvasBounds| {
                    let cs = cursor_state_clone.lock().unwrap();

                    if !cs.visible {
                        return;
                    }

                    let opacity = cs.current_opacity();
                    if opacity < 0.01 {
                        return;
                    }

                    let color = blinc_core::Color::rgba(
                        cs.color.r,
                        cs.color.g,
                        cs.color.b,
                        cs.color.a * opacity,
                    );

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

            Some(cursor_canvas)
        } else {
            if let Ok(mut cs) = cursor_state_for_canvas.lock() {
                cs.visible = false;
            }
            None
        };

        // Build text content - left-aligned column of text lines
        // Use relative positioning to allow cursor absolute positioning within
        let mut text_content = div().flex_col().justify_start().items_start().relative();

        if data.is_empty() {
            // Use state's placeholder if available, otherwise fall back to config
            let placeholder = if !data.placeholder.is_empty() {
                &data.placeholder
            } else {
                &config.placeholder
            };

            text_content = text_content.child(
                div().h(line_height).flex_row().items_center().child(
                    text(placeholder)
                        .size(config.font_size)
                        .color(text_color)
                        .text_left()
                        .no_wrap(),
                ),
            );
        } else {
            for (_line_idx, line) in data.lines.iter().enumerate() {
                let line_text = if line.is_empty() { " " } else { line.as_str() };
                text_content = text_content.child(
                    div().h(line_height).flex_row().items_center().child(
                        text(line_text)
                            .size(config.font_size)
                            .color(text_color)
                            .text_left()
                            .no_wrap(),
                    ),
                );
            }
        }

        // Add cursor inside text_content so it scrolls with the text
        if let Some(cursor) = cursor_canvas_opt {
            text_content = text_content.child(cursor);
        }

        // Wrap text content in scroll container with shared physics
        // This provides proper scroll handling and clipping
        // TextArea scroll doesn't use bounce animation - just hard stops at edges
        let scrollable_content = Scroll::with_physics(Arc::clone(&data.scroll_physics))
            .direction(ScrollDirection::Vertical)
            .no_bounce()
            .w_full()
            .h_full()
            .child(text_content);

        let inner_content = div()
            .w_full()
            .h_full()
            .bg(bg)
            .rounded(config.corner_radius - 1.0)
            .padding_y_px(config.padding_y)
            .padding_x_px(config.padding_x)
            .flex_col()
            .justify_start()
            .items_start()
            .child(scrollable_content);

        // Build outer container (border box)
        div()
            .bg(border)
            .rounded(config.corner_radius)
            .p(config.border_width)
            .child(inner_content)
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        let placeholder = text.into();
        self.config.lock().unwrap().placeholder = placeholder.clone();
        if let Ok(mut s) = self.state.lock() {
            s.placeholder = placeholder;
        }
        self
    }

    /// Update cached scroll dimensions from config
    /// This must be called whenever config values that affect scroll calculation change
    fn update_scroll_dimensions(&self) {
        let cfg = self.config.lock().unwrap();
        let line_height = cfg.font_size * cfg.line_height;
        let viewport_height = cfg.effective_height() - cfg.padding_y * 2.0 - cfg.border_width * 2.0;
        let viewport_width = cfg.effective_width() - cfg.padding_x * 2.0 - cfg.border_width * 2.0;
        drop(cfg);

        if let Ok(mut s) = self.state.lock() {
            s.line_height = line_height;
            s.viewport_height = viewport_height;
            // Update scroll physics viewport dimensions
            if let Ok(mut physics) = s.scroll_physics.lock() {
                physics.viewport_height = viewport_height;
                physics.viewport_width = viewport_width;
            }
        }
    }

    /// Set number of visible rows (like HTML textarea rows attribute)
    pub fn rows(mut self, rows: usize) -> Self {
        let height = {
            let mut cfg = self.config.lock().unwrap();
            cfg.rows = Some(rows);
            cfg.effective_height()
        };
        self.inner = std::mem::take(&mut self.inner).h(height);
        self.update_scroll_dimensions();
        self
    }

    /// Set number of visible columns (like HTML textarea cols attribute)
    pub fn cols(mut self, cols: usize) -> Self {
        let width = {
            let mut cfg = self.config.lock().unwrap();
            cfg.cols = Some(cols);
            cfg.effective_width()
        };
        self.inner = std::mem::take(&mut self.inner).w(width);
        self
    }

    /// Set both rows and cols
    pub fn text_size(mut self, rows: usize, cols: usize) -> Self {
        let (width, height) = {
            let mut cfg = self.config.lock().unwrap();
            cfg.rows = Some(rows);
            cfg.cols = Some(cols);
            (cfg.effective_width(), cfg.effective_height())
        };
        self.inner = std::mem::take(&mut self.inner).w(width).h(height);
        self.update_scroll_dimensions();
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.config.lock().unwrap().font_size = size;
        self.update_scroll_dimensions();
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.lock().unwrap().disabled = disabled;
        if let Ok(mut s) = self.state.lock() {
            s.disabled = disabled;
            if disabled {
                s.visual = TextFieldState::Disabled;
            }
        }
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max: usize) -> Self {
        self.config.lock().unwrap().max_length = max;
        self
    }

    // =========================================================================
    // Builder methods that return Self (shadow Div methods for fluent API)
    // =========================================================================

    pub fn w(mut self, px: f32) -> Self {
        {
            let mut cfg = self.config.lock().unwrap();
            cfg.width = px;
            cfg.cols = None;
        }
        self.inner = std::mem::take(&mut self.inner).w(px);
        self
    }

    pub fn h(mut self, px: f32) -> Self {
        {
            let mut cfg = self.config.lock().unwrap();
            cfg.height = px;
            cfg.rows = None;
        }
        self.inner = std::mem::take(&mut self.inner).h(px);
        self.update_scroll_dimensions();
        self
    }

    pub fn size(mut self, w: f32, h: f32) -> Self {
        {
            let mut cfg = self.config.lock().unwrap();
            cfg.width = w;
            cfg.height = h;
            cfg.cols = None;
            cfg.rows = None;
        }
        self.inner = std::mem::take(&mut self.inner).size(w, h);
        self.update_scroll_dimensions();
        self
    }

    pub fn square(mut self, size: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).square(size);
        self
    }

    pub fn w_full(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).w_full();
        self
    }

    pub fn h_full(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).h_full();
        self
    }

    pub fn w_fit(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).w_fit();
        self
    }

    pub fn h_fit(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).h_fit();
        self
    }

    pub fn p(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).p(px);
        self
    }

    pub fn px(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).px(px);
        self
    }

    pub fn py(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).py(px);
        self
    }

    pub fn m(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).m(px);
        self
    }

    pub fn mx(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).mx(px);
        self
    }

    pub fn my(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).my(px);
        self
    }

    pub fn gap(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).gap(px);
        self
    }

    pub fn flex_row(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).flex_row();
        self
    }

    pub fn flex_col(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).flex_col();
        self
    }

    pub fn flex_grow(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).flex_grow();
        self
    }

    pub fn items_center(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).items_center();
        self
    }

    pub fn items_start(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).items_start();
        self
    }

    pub fn items_end(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).items_end();
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).justify_center();
        self
    }

    pub fn justify_start(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).justify_start();
        self
    }

    pub fn justify_end(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).justify_end();
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).justify_between();
        self
    }

    pub fn bg(mut self, color: impl Into<blinc_core::Brush>) -> Self {
        self.inner = std::mem::take(&mut self.inner).bg(color);
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.config.lock().unwrap().corner_radius = radius;
        self.inner = std::mem::take(&mut self.inner).rounded(radius);
        self
    }

    pub fn shadow(mut self, shadow: blinc_core::Shadow) -> Self {
        self.inner = std::mem::take(&mut self.inner).shadow(shadow);
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

    pub fn shadow_lg(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).shadow_lg();
        self
    }

    pub fn transform(mut self, transform: blinc_core::Transform) -> Self {
        self.inner = std::mem::take(&mut self.inner).transform(transform);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).opacity(opacity);
        self
    }

    pub fn overflow_clip(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).overflow_clip();
        self
    }

    pub fn child(mut self, child: impl ElementBuilder + 'static) -> Self {
        self.inner = std::mem::take(&mut self.inner).child(child);
        self
    }

    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: ElementBuilder + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).children(children);
        self
    }

    // Event handlers
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_click(handler);
        self
    }

    pub fn on_hover_enter<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_hover_enter(handler);
        self
    }

    pub fn on_hover_leave<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_hover_leave(handler);
        self
    }

    pub fn on_mouse_down<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_mouse_down(handler);
        self
    }

    pub fn on_mouse_up<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_mouse_up(handler);
        self
    }

    pub fn on_focus<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_focus(handler);
        self
    }

    pub fn on_blur<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_blur(handler);
        self
    }

    pub fn on_key_down<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_key_down(handler);
        self
    }

    pub fn on_key_up<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_key_up(handler);
        self
    }

    pub fn on_scroll<F>(mut self, handler: F) -> Self
    where
        F: Fn(&crate::event_handler::EventContext) + Send + Sync + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).on_scroll(handler);
        self
    }
}

/// Create a ready-to-use multi-line text area
///
/// The text area inherits ALL Div methods, so you have full layout control.
///
/// # Example
///
/// ```ignore
/// let state = text_area_state_with_placeholder("Enter message...");
/// text_area(&state)
///     .rows(4)
///     .w(400.0)
///     .rounded(12.0)
///     .shadow_sm()
/// ```
pub fn text_area(state: &SharedTextAreaState) -> TextArea {
    TextArea::new(state)
}

impl ElementBuilder for TextArea {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        // Set base render props for incremental updates
        // Note: callback and handlers are registered in new() so they're available for incremental diff
        {
            let shared_state = self.inner.shared_state();
            let mut shared = shared_state.lock().unwrap();
            shared.base_render_props = Some(self.inner.inner_render_props());
        }

        // Build the inner Stateful
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
        ElementBuilder::event_handlers(&self.inner)
    }

    fn layout_style(&self) -> Option<&taffy::Style> {
        self.inner.layout_style()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_area_state_insert() {
        let mut state = TextAreaState::new();
        state.insert("hello");
        assert_eq!(state.value(), "hello");

        state.insert_newline();
        state.insert("world");
        assert_eq!(state.value(), "hello\nworld");
        assert_eq!(state.line_count(), 2);
    }

    #[test]
    fn test_text_area_state_delete() {
        let mut state = TextAreaState::with_value("hello\nworld");
        state.cursor = TextPosition::new(1, 5);

        state.delete_backward();
        assert_eq!(state.value(), "hello\nworl");

        state.cursor = TextPosition::new(1, 0);
        state.delete_backward();
        assert_eq!(state.value(), "helloworl");
        assert_eq!(state.line_count(), 1);
    }

    #[test]
    fn test_text_area_state_navigation() {
        let mut state = TextAreaState::with_value("line1\nline2\nline3");
        state.cursor = TextPosition::new(1, 3);

        state.move_up(false);
        assert_eq!(state.cursor, TextPosition::new(0, 3));

        state.move_down(false);
        assert_eq!(state.cursor, TextPosition::new(1, 3));

        state.move_to_line_start(false);
        assert_eq!(state.cursor, TextPosition::new(1, 0));

        state.move_to_line_end(false);
        assert_eq!(state.cursor, TextPosition::new(1, 5));
    }

    #[test]
    fn test_text_area_state_selection() {
        let mut state = TextAreaState::with_value("hello\nworld");

        state.select_all();
        assert_eq!(state.selected_text(), Some("hello\nworld".to_string()));

        state.insert("new");
        assert_eq!(state.value(), "new");
        assert_eq!(state.line_count(), 1);
    }
}
