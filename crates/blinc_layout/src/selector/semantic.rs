#[cfg(test)]
use std::sync::{Mutex, MutexGuard, Once};

use blinc_platform::AccessibilityRole;

use crate::renderer::{ElementType, RenderTree};
use crate::tree::LayoutNodeId;

#[cfg(test)]
use blinc_theme::ThemeState;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticLocator {
    role: Option<AccessibilityRole>,
    text: Option<String>,
    label: Option<String>,
    placeholder: Option<String>,
    tag: Option<String>,
    within: Option<String>,
    nth: Option<usize>,
}

impl SemanticLocator {
    pub fn role(role: AccessibilityRole) -> Self {
        Self {
            role: Some(role),
            ..Self::default()
        }
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::default()
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn within(mut self, scope_id: impl Into<String>) -> Self {
        self.within = Some(scope_id.into());
        self
    }

    pub fn nth(mut self, index: usize) -> Self {
        self.nth = Some(index);
        self
    }

    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if let Some(role) = self.role {
            parts.push(format!("role={role:?}"));
        }
        if let Some(text) = self.text.as_deref() {
            parts.push(format!("text={text:?}"));
        }
        if let Some(label) = self.label.as_deref() {
            parts.push(format!("label={label:?}"));
        }
        if let Some(placeholder) = self.placeholder.as_deref() {
            parts.push(format!("placeholder={placeholder:?}"));
        }
        if let Some(tag) = self.tag.as_deref() {
            parts.push(format!("tag={tag:?}"));
        }
        if let Some(scope) = self.within.as_deref() {
            parts.push(format!("within={scope:?}"));
        }
        if let Some(index) = self.nth {
            parts.push(format!("nth={index}"));
        }
        parts.join(", ")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticLocatorResolution {
    pub query: String,
    pub matched_node_id: Option<LayoutNodeId>,
    pub matched_target: Option<String>,
    pub candidate_targets: Vec<String>,
    pub failure_reason: Option<String>,
}

impl SemanticLocatorResolution {
    pub fn is_ambiguous(&self) -> bool {
        self.failure_reason.as_deref() == Some("ambiguous_match")
    }
}

pub fn resolve_semantic_locator(
    tree: &RenderTree,
    locator: &SemanticLocator,
) -> SemanticLocatorResolution {
    let query = locator.describe();
    let Some(root) = tree.root() else {
        return record_resolution(SemanticLocatorResolution {
            query,
            matched_node_id: None,
            matched_target: None,
            candidate_targets: Vec::new(),
            failure_reason: Some("empty_tree".to_string()),
        });
    };

    let scope_root = match locator.within.as_deref() {
        Some(scope_id) => match tree.query_by_id(scope_id) {
            Some(node_id) => node_id,
            None => {
                return record_resolution(SemanticLocatorResolution {
                    query,
                    matched_node_id: None,
                    matched_target: None,
                    candidate_targets: Vec::new(),
                    failure_reason: Some("within_scope_not_found".to_string()),
                });
            }
        },
        None => root,
    };

    let mut nodes = Vec::new();
    collect_subtree_nodes(tree, scope_root, &mut nodes);

    let matched_nodes = nodes
        .into_iter()
        .filter(|&node_id| matches_locator(tree, node_id, locator))
        .collect::<Vec<_>>();
    let matched_nodes = preferred_text_only_matches(tree, locator, matched_nodes);
    let candidate_targets = matched_nodes
        .iter()
        .map(|&node_id| target_label(tree, node_id))
        .collect::<Vec<_>>();

    let matched_node_id = match locator.nth {
        Some(index) => matched_nodes.get(index).copied(),
        None if matched_nodes.len() == 1 => matched_nodes.first().copied(),
        None => None,
    };
    let failure_reason = if matched_node_id.is_some() {
        None
    } else if matched_nodes.is_empty() {
        Some("no_match".to_string())
    } else if locator.nth.is_some() {
        Some("nth_out_of_range".to_string())
    } else {
        Some("ambiguous_match".to_string())
    };

    record_resolution(SemanticLocatorResolution {
        query,
        matched_node_id,
        matched_target: matched_node_id.map(|node_id| target_label(tree, node_id)),
        candidate_targets,
        failure_reason,
    })
}

fn preferred_text_only_matches(
    tree: &RenderTree,
    locator: &SemanticLocator,
    matched_nodes: Vec<LayoutNodeId>,
) -> Vec<LayoutNodeId> {
    if !is_text_only_locator(locator) || matched_nodes.len() <= 1 {
        return matched_nodes;
    }

    let accessible = matched_nodes
        .iter()
        .copied()
        .filter(|&node_id| tree.layout().accessibility_metadata(node_id).is_some())
        .collect::<Vec<_>>();
    if !accessible.is_empty() {
        return prune_ancestor_candidates(tree, accessible);
    }

    let actionable = matched_nodes
        .iter()
        .copied()
        .filter(|&node_id| is_actionable_text_candidate(tree, node_id))
        .collect::<Vec<_>>();
    if actionable.is_empty() {
        matched_nodes
    } else {
        prune_ancestor_candidates(tree, actionable)
    }
}

fn is_text_only_locator(locator: &SemanticLocator) -> bool {
    locator.text.is_some()
        && locator.role.is_none()
        && locator.label.is_none()
        && locator.placeholder.is_none()
        && locator.tag.is_none()
}

fn is_actionable_text_candidate(tree: &RenderTree, node_id: LayoutNodeId) -> bool {
    tree.layout().accessibility_metadata(node_id).is_some()
        || tree.element_registry().get_id(node_id).is_some()
}

fn prune_ancestor_candidates(
    tree: &RenderTree,
    candidates: Vec<LayoutNodeId>,
) -> Vec<LayoutNodeId> {
    candidates
        .iter()
        .copied()
        .filter(|&candidate| {
            !candidates
                .iter()
                .copied()
                .any(|other| other != candidate && is_ancestor_candidate(tree, candidate, other))
        })
        .collect()
}

fn is_ancestor_candidate(tree: &RenderTree, ancestor: LayoutNodeId, node_id: LayoutNodeId) -> bool {
    let mut current = tree.element_registry().get_parent(node_id);
    while let Some(parent) = current {
        if parent == ancestor {
            return true;
        }
        current = tree.element_registry().get_parent(parent);
    }
    false
}

fn record_resolution(resolution: SemanticLocatorResolution) -> SemanticLocatorResolution {
    record_locator_resolution(
        &resolution.query,
        resolution.matched_target.as_deref(),
        &resolution.candidate_targets,
        resolution.failure_reason.as_deref(),
    );
    resolution
}

#[cfg(feature = "recorder")]
fn record_locator_resolution(
    query: &str,
    matched_target: Option<&str>,
    candidate_targets: &[String],
    failure_reason: Option<&str>,
) {
    crate::recorder_bridge::record_locator_resolution(
        query,
        matched_target,
        candidate_targets,
        failure_reason,
    );
}

#[cfg(not(feature = "recorder"))]
fn record_locator_resolution(
    query: &str,
    matched_target: Option<&str>,
    candidate_targets: &[String],
    failure_reason: Option<&str>,
) {
    let _ = (query, matched_target, candidate_targets, failure_reason);
}

fn collect_subtree_nodes(tree: &RenderTree, node_id: LayoutNodeId, nodes: &mut Vec<LayoutNodeId>) {
    nodes.push(node_id);
    for child in tree.layout().children(node_id) {
        collect_subtree_nodes(tree, child, nodes);
    }
}

fn target_label(tree: &RenderTree, node_id: LayoutNodeId) -> String {
    tree.element_registry()
        .get_id(node_id)
        .unwrap_or_else(|| format!("node#{}", node_id.to_raw()))
}

fn matches_locator(tree: &RenderTree, node_id: LayoutNodeId, locator: &SemanticLocator) -> bool {
    if let Some(tag) = locator.tag.as_deref() {
        let Some(actual_tag) = tree.element_registry().get_element_type(node_id) else {
            return false;
        };
        if !normalized_eq(&actual_tag, tag) {
            return false;
        }
    }

    let accessibility = tree.layout().accessibility_metadata(node_id);
    if let Some(role) = locator.role {
        let Some(metadata) = accessibility.as_ref() else {
            return false;
        };
        if metadata.role != role {
            return false;
        }
    }

    if let Some(label) = locator.label.as_deref() {
        let Some(metadata) = accessibility.as_ref() else {
            return false;
        };
        let Some(name) = metadata.name.as_deref() else {
            return false;
        };
        if matches!(
            metadata.role,
            AccessibilityRole::TextInput | AccessibilityRole::TextArea
        ) {
            if metadata
                .description
                .as_deref()
                .is_some_and(|description| normalized_eq(description, name))
            {
                return false;
            }
            if tree
                .element_registry()
                .get_id(node_id)
                .as_deref()
                .is_some_and(|element_id| normalized_eq(element_id, name))
            {
                return false;
            }
        }
        if !normalized_contains(name, label) {
            return false;
        }
    }

    if let Some(placeholder) = locator.placeholder.as_deref() {
        let Some(metadata) = accessibility.as_ref() else {
            return false;
        };
        let Some(actual_placeholder) = metadata.description.as_deref() else {
            return false;
        };
        if !normalized_contains(actual_placeholder, placeholder) {
            return false;
        }
    }

    if let Some(text) = locator.text.as_deref() {
        let Some(actual_text) = rendered_text(tree, node_id) else {
            return false;
        };
        if !normalized_contains(&actual_text, text) {
            return false;
        }
    }

    true
}

fn rendered_text(tree: &RenderTree, node_id: LayoutNodeId) -> Option<String> {
    let mut parts = Vec::new();
    collect_rendered_text(tree, node_id, &mut parts);
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn normalized_contains(actual: &str, expected: &str) -> bool {
    let actual = actual.trim().to_lowercase();
    let expected = expected.trim().to_lowercase();
    actual.contains(&expected)
}

fn normalized_eq(actual: &str, expected: &str) -> bool {
    actual.trim().to_lowercase() == expected.trim().to_lowercase()
}

fn collect_rendered_text(tree: &RenderTree, node_id: LayoutNodeId, parts: &mut Vec<String>) {
    match tree.get_render_node(node_id).map(|node| &node.element_type) {
        Some(ElementType::Text(data)) => parts.push(data.content.clone()),
        Some(ElementType::StyledText(data)) => parts.push(data.content.clone()),
        _ => {}
    }

    for child in tree.layout().children(node_id) {
        collect_rendered_text(tree, child, parts);
    }
}

#[cfg(test)]
fn ensure_theme() {
    static INIT: Once = Once::new();
    INIT.call_once(ThemeState::init_default);
}

#[cfg(test)]
fn semantic_test_guard() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().expect("semantic test lock")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::div::div;
    use crate::stateful::{ButtonState, Stateful};
    use crate::text::text;
    use crate::widgets::{
        blur_all_text_inputs, button, text_input, text_input_state,
        text_input_state_with_placeholder,
    };

    #[test]
    fn semantic_query_matches_role_and_label() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let button_state = Stateful::new(ButtonState::Idle).shared_state();
        let input = text_input_state();
        let ui = div()
            .id("auth-form")
            .flex_col()
            .child(button(button_state, "Submit").id("submit-button"))
            .child(
                text_input(&input)
                    .id("email-input")
                    .placeholder("Email Address"),
            );

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(480.0, 240.0);

        let button_resolution = resolve_semantic_locator(
            &tree,
            &SemanticLocator::role(AccessibilityRole::Button).with_label("Submit"),
        );
        assert_eq!(
            button_resolution.matched_target.as_deref(),
            Some("submit-button")
        );
        assert_eq!(button_resolution.failure_reason, None);

        let input_resolution = resolve_semantic_locator(
            &tree,
            &SemanticLocator::role(AccessibilityRole::TextInput)
                .with_placeholder("Email")
                .within("auth-form"),
        );
        assert!(input_resolution.matched_node_id.is_some());
        assert_eq!(input_resolution.candidate_targets.len(), 1);
        assert_eq!(input_resolution.failure_reason, None);
    }

    #[test]
    fn semantic_query_reports_ambiguity_when_multiple_nodes_match() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let primary = Stateful::new(ButtonState::Idle).shared_state();
        let secondary = Stateful::new(ButtonState::Idle).shared_state();
        let ui = div()
            .flex_col()
            .child(button(primary, "Save").id("save-primary"))
            .child(button(secondary, "Save").id("save-secondary"));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(480.0, 240.0);

        let ambiguous =
            resolve_semantic_locator(&tree, &SemanticLocator::role(AccessibilityRole::Button));
        assert!(ambiguous.is_ambiguous());
        assert_eq!(ambiguous.matched_target, None);
        assert_eq!(
            ambiguous.candidate_targets,
            vec!["save-primary".to_string(), "save-secondary".to_string()]
        );

        let second = resolve_semantic_locator(
            &tree,
            &SemanticLocator::role(AccessibilityRole::Button).nth(1),
        );
        assert_eq!(second.matched_target.as_deref(), Some("save-secondary"));
        assert_eq!(second.failure_reason, None);
    }

    #[test]
    fn semantic_query_matches_unicode_text_case_insensitively() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let button_state = Stateful::new(ButtonState::Idle).shared_state();
        let ui = div().child(button(button_state, "Ångström").id("unicode-button"));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(320.0, 120.0);

        let resolution = resolve_semantic_locator(
            &tree,
            &SemanticLocator::role(AccessibilityRole::Button).with_label("ång"),
        );
        assert_eq!(resolution.matched_target.as_deref(), Some("unicode-button"));
        assert_eq!(resolution.failure_reason, None);
    }

    #[test]
    fn semantic_query_matches_visible_descendant_text() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let button_state = Stateful::new(ButtonState::Idle).shared_state();
        let ui = div().child(button(button_state, "Submit").id("submit-button"));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(320.0, 120.0);

        let resolution = resolve_semantic_locator(
            &tree,
            &SemanticLocator::role(AccessibilityRole::Button).with_text("Submit"),
        );
        assert_eq!(resolution.matched_target.as_deref(), Some("submit-button"));
        assert_eq!(resolution.failure_reason, None);
    }

    #[test]
    fn semantic_text_only_query_prefers_actionable_node_over_ancestor_text_matches() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let button_state = Stateful::new(ButtonState::Idle).shared_state();
        let ui = div()
            .id("screen")
            .child(button(button_state, "Submit").id("submit-button"));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(320.0, 120.0);

        let resolution = resolve_semantic_locator(&tree, &SemanticLocator::text("Submit"));
        assert_eq!(resolution.matched_target.as_deref(), Some("submit-button"));
        assert_eq!(resolution.failure_reason, None);
    }

    #[test]
    fn semantic_text_only_query_prefers_deepest_accessible_match() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let ui = div()
            .id("outer-button")
            .child(div().id("inner-checkbox").child(text("Submit")));

        let mut tree = RenderTree::from_element(&ui);
        let outer_button = tree
            .query_by_id("outer-button")
            .expect("outer button should resolve");
        let inner_checkbox = tree
            .query_by_id("inner-checkbox")
            .expect("inner checkbox should resolve");
        tree.layout_tree.set_accessibility_metadata(
            outer_button,
            crate::accessibility::AccessibilityMetadata::new(AccessibilityRole::Button)
                .with_name(Some("Submit".to_string()))
                .with_focusable(true),
        );
        tree.layout_tree.set_accessibility_metadata(
            inner_checkbox,
            crate::accessibility::AccessibilityMetadata::new(AccessibilityRole::Checkbox)
                .with_name(Some("Submit".to_string()))
                .with_focusable(true),
        );
        tree.compute_layout(320.0, 120.0);

        let resolution = resolve_semantic_locator(&tree, &SemanticLocator::text("Submit"));
        assert_eq!(resolution.matched_node_id, Some(inner_checkbox));
        assert_eq!(resolution.failure_reason, None);
    }

    #[test]
    fn semantic_query_tag_matching_is_exact() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let input = text_input_state_with_placeholder("Email Address");
        let ui = div().child(
            text_input(&input)
                .id("email-input")
                .placeholder("Email Address"),
        );

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(320.0, 120.0);

        let resolution =
            resolve_semantic_locator(&tree, &SemanticLocator::default().with_tag("text"));
        assert_eq!(resolution.matched_target, None);
        assert_eq!(resolution.failure_reason.as_deref(), Some("no_match"));
    }

    #[test]
    fn placeholder_query_does_not_match_id_when_placeholder_is_missing() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let with_placeholder = text_input_state_with_placeholder("Email Address");
        let without_placeholder = text_input_state();
        let ui = div()
            .flex_col()
            .child(
                text_input(&with_placeholder)
                    .id("with-placeholder")
                    .placeholder("Email Address"),
            )
            .child(text_input(&without_placeholder).id("login.email"));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(480.0, 240.0);

        let resolution = resolve_semantic_locator(
            &tree,
            &SemanticLocator::role(AccessibilityRole::TextInput).with_placeholder("login.email"),
        );
        assert_eq!(resolution.matched_target, None);
        assert_eq!(resolution.failure_reason.as_deref(), Some("no_match"));
    }

    #[test]
    fn label_query_does_not_match_text_input_placeholder_or_id() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let with_placeholder = text_input_state_with_placeholder("Email Address");
        let without_placeholder = text_input_state();
        let ui = div()
            .flex_col()
            .child(
                text_input(&with_placeholder)
                    .id("with-placeholder")
                    .placeholder("Email Address"),
            )
            .child(text_input(&without_placeholder).id("login.email"));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(480.0, 240.0);

        let resolution = resolve_semantic_locator(
            &tree,
            &SemanticLocator::role(AccessibilityRole::TextInput).with_label("Email"),
        );
        assert_eq!(resolution.matched_target, None);
        assert_eq!(resolution.failure_reason.as_deref(), Some("no_match"));
    }

    #[test]
    fn text_input_id_resolves_to_input_widget_node() {
        let _guard = semantic_test_guard();
        ensure_theme();
        blur_all_text_inputs();

        let input = text_input_state_with_placeholder("Email Address");
        let ui = div().child(
            text_input(&input)
                .id("login.email")
                .placeholder("Email Address"),
        );

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(320.0, 120.0);

        let node_id = tree
            .query_by_id("login.email")
            .expect("text input id should resolve");
        assert_eq!(
            tree.element_registry().get_element_type(node_id).as_deref(),
            Some("input")
        );
    }
}
