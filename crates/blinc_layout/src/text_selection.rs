//! Global text selection state for clipboard support
//!
//! Provides a centralized location to track what text is currently selected
//! across all text input widgets. This enables clipboard operations (copy/cut/paste)
//! to work with any focused text input.

use std::sync::{Arc, Mutex, OnceLock};

/// Source of the selected text
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionSource {
    /// Selection from a text input widget
    TextInput,
    /// Selection from a text area widget
    TextArea,
    /// Selection from static/label text
    StaticText,
}

/// Global text selection state
#[derive(Debug, Clone, Default)]
pub struct TextSelection {
    /// The currently selected text (if any)
    pub text: Option<String>,
    /// Source widget type
    pub source: Option<SelectionSource>,
    /// Whether the selection can be cut (vs just copied)
    pub can_cut: bool,
}

impl TextSelection {
    /// Create an empty selection
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a new selection
    pub fn new(text: String, source: SelectionSource, can_cut: bool) -> Self {
        Self {
            text: Some(text),
            source: Some(source),
            can_cut,
        }
    }

    /// Check if there's any text selected
    pub fn has_selection(&self) -> bool {
        self.text.as_ref().is_some_and(|t| !t.is_empty())
    }

    /// Get the selected text
    pub fn selected_text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.text = None;
        self.source = None;
        self.can_cut = false;
    }
}

/// Thread-safe handle to the global text selection state
pub type SharedTextSelection = Arc<Mutex<TextSelection>>;

/// Get the global text selection state
///
/// This is a singleton that persists for the lifetime of the application.
/// Use this to check what text is currently selected for clipboard operations.
pub fn global_selection() -> SharedTextSelection {
    static GLOBAL_SELECTION: OnceLock<SharedTextSelection> = OnceLock::new();
    Arc::clone(GLOBAL_SELECTION.get_or_init(|| Arc::new(Mutex::new(TextSelection::empty()))))
}

/// Set the global text selection
///
/// Call this when a text input's selection changes.
pub fn set_selection(text: String, source: SelectionSource, can_cut: bool) {
    let selection = global_selection();
    let mut guard = selection.lock().unwrap();
    *guard = TextSelection::new(text, source, can_cut);
}

/// Clear the global text selection
///
/// Call this when focus leaves a text input or selection is cleared.
pub fn clear_selection() {
    let selection = global_selection();
    let mut guard = selection.lock().unwrap();
    guard.clear();
}

/// Get the currently selected text (convenience function)
pub fn get_selected_text() -> Option<String> {
    let selection = global_selection();
    let guard = selection.lock().unwrap();
    guard.text.clone()
}

/// Check if the current selection can be cut
pub fn can_cut_selection() -> bool {
    let selection = global_selection();
    let guard = selection.lock().unwrap();
    guard.can_cut
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_empty() {
        let sel = TextSelection::empty();
        assert!(!sel.has_selection());
        assert!(sel.selected_text().is_none());
    }

    #[test]
    fn test_selection_with_text() {
        let sel = TextSelection::new("hello".to_string(), SelectionSource::TextInput, true);
        assert!(sel.has_selection());
        assert_eq!(sel.selected_text(), Some("hello"));
        assert!(sel.can_cut);
    }

    #[test]
    fn test_global_selection() {
        // Clear any previous state
        clear_selection();
        assert!(get_selected_text().is_none());

        // Set selection
        set_selection("test text".to_string(), SelectionSource::TextInput, true);
        assert_eq!(get_selected_text(), Some("test text".to_string()));
        assert!(can_cut_selection());

        // Clear selection
        clear_selection();
        assert!(get_selected_text().is_none());
    }
}
