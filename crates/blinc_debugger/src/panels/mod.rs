//! Debugger panels
//!
//! The main UI panels as specified in the implementation plan:
//! - Tree Panel: Element tree with diff visualization
//! - Preview Panel: Live/recorded UI preview
//! - Inspector Panel: Selected element properties
//! - Timeline Panel: Event timeline with scrubber

pub mod inspector_panel;
pub mod preview_panel;
pub mod timeline_panel;
pub mod tree_panel;

pub use inspector_panel::InspectorPanel;
pub use preview_panel::{PreviewConfig, PreviewPanel};
pub use timeline_panel::{TimelinePanel, TimelinePanelState};
pub use tree_panel::{TreePanel, TreePanelState};
