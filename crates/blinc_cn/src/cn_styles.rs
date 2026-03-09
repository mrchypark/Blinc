//! Default CSS stylesheet for blinc_cn components.
//!
//! All visual properties use `var()` to reference theme tokens,
//! making everything overridable via CSS. Component-level variables
//! (e.g., `--cn-button-primary-bg`) provide targeted override points
//! that fall back to theme tokens.
//!
//! # Usage
//!
//! ```ignore
//! // Register default styles before user CSS
//! blinc_cn::register_cn_styles(ctx);
//!
//! // User CSS can then override:
//! ctx.add_css(r#"
//!     .cn-button--primary { background: #ff6600; }
//!     .cn-card { border-radius: 0; }
//! "#);
//! ```

/// Default CSS for all blinc_cn components.
///
/// Uses `var(--theme-token)` for all color references.
/// Each component defines `var(--cn-component-prop, var(--fallback))` for overridability.
pub const CN_STYLES: &str = r#"
/* ============================================================================
   Utilities
   ============================================================================ */

.cn-truncate {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.cn-decorative {
    pointer-events: none;
    cursor: default;
}

/* ============================================================================
   Button
   ============================================================================ */

/* Button: visual states (hover, active, disabled) handled by Stateful FSM.
   CSS defines border-radius and padding per size. User CSS can override these classes. */
.cn-button {
    border-radius: var(--cn-button-radius, var(--control-radius-md));
    corner-shape: var(--cn-button-corner-shape, var(--control-corner-shape));
    transition: corner-shape 180ms, box-shadow 180ms;
}
.cn-button:hover {
    corner-shape: var(--cn-button-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-button--primary { }
.cn-button--secondary { }
.cn-button--destructive { }
.cn-button--outline { }
.cn-button--ghost { }
.cn-button--link {
    text-decoration: underline;
    text-decoration-color: var(--primary);
    text-decoration-thickness: 1.5px;
}
.cn-button--disabled { }
.cn-button--sm { border-radius: var(--cn-button-radius-sm, var(--control-radius-sm)); }
.cn-button--md { border-radius: var(--cn-button-radius-md, var(--control-radius-md)); }
.cn-button--lg { border-radius: var(--cn-button-radius-lg, var(--control-radius-lg)); }
.cn-button--icon { border-radius: var(--cn-button-radius-icon, var(--control-radius-md)); }
.cn-button__label { max-width: 100%; }

/* ============================================================================
   Card
   ============================================================================ */

.cn-card {
    background: var(--cn-card-bg, var(--surface));
    border: 1px solid var(--cn-card-border, var(--border));
    border-radius: var(--cn-card-radius, var(--container-radius));
    padding: var(--cn-card-padding, var(--container-padding));
    gap: var(--cn-card-gap, var(--container-section-gap));
    corner-shape: var(--cn-card-corner-shape, var(--container-corner-shape));
    transition: corner-shape 180ms, box-shadow 180ms;
}

.cn-card-header {
    gap: var(--cn-card-header-gap, var(--container-header-gap));
}

.cn-card-footer {
    gap: var(--cn-card-footer-gap, var(--container-footer-gap));
}

/* ============================================================================
   Badge
   ============================================================================ */

.cn-badge {
    border-radius: var(--cn-badge-radius, var(--compact-badge-radius));
    font-size: var(--cn-badge-font-size, var(--type-badge));
    padding: var(--cn-badge-py, var(--compact-badge-py)) var(--cn-badge-px, var(--compact-badge-px));
    corner-shape: var(--cn-badge-corner-shape, 2);
}
.cn-badge--default {
    background: var(--primary);
    color: var(--text-inverse);
}
.cn-badge--secondary {
    background: var(--secondary);
    color: var(--text-inverse);
}
.cn-badge--success {
    background: var(--success);
    color: var(--text-inverse);
}
.cn-badge--warning {
    background: var(--warning);
    color: var(--text-inverse);
}
.cn-badge--destructive {
    background: var(--error);
    color: var(--text-inverse);
}
.cn-badge--outline {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-primary);
}

/* ============================================================================
   Alert
   ============================================================================ */

.cn-alert {
    background: var(--cn-alert-bg, var(--surface));
    border: 1px solid var(--cn-alert-border, var(--border));
    border-radius: var(--cn-alert-radius, var(--container-radius));
    color: var(--text-primary);
    font-size: var(--cn-alert-font-size, var(--type-body-md));
    padding: var(--cn-alert-padding, var(--container-padding-compact));
    corner-shape: var(--cn-alert-corner-shape, var(--container-corner-shape));
}
.cn-alert-box {
    background: var(--cn-alert-bg, var(--surface));
    border: 1px solid var(--cn-alert-border, var(--border));
    border-radius: var(--cn-alert-radius, var(--container-radius));
    color: var(--text-primary);
    font-size: var(--cn-alert-font-size, var(--type-body-md));
    padding: var(--cn-alert-padding, var(--container-padding-compact));
    gap: var(--cn-alert-gap, var(--overlay-gap));
    corner-shape: var(--cn-alert-corner-shape, var(--container-corner-shape));
}
.cn-alert--success {
    background: var(--success-bg);
    border-color: var(--success);
    color: var(--success);
}
.cn-alert--warning {
    background: var(--warning-bg);
    border-color: var(--warning);
    color: var(--warning);
}
.cn-alert--error {
    background: var(--error-bg);
    border-color: var(--error);
    color: var(--error);
}
.cn-alert--info {
    background: var(--info-bg);
    border-color: var(--info);
    color: var(--info);
}

/* ============================================================================
   Separator
   ============================================================================ */

.cn-separator {
    background: var(--cn-separator-color, var(--border));
}

/* ============================================================================
   Skeleton
   ============================================================================ */

.cn-skeleton {
    background: var(--cn-skeleton-bg, var(--surface-elevated));
    border-radius: var(--cn-skeleton-radius, var(--control-radius-sm));
    corner-shape: var(--cn-skeleton-corner-shape, var(--control-corner-shape));
    mix-blend-mode: overlay;
}

/* ============================================================================
   Input
   ============================================================================ */

.cn-input {
    background: var(--cn-input-bg, var(--input-bg));
    border: 1px solid var(--cn-input-border, var(--border));
    border-radius: var(--cn-input-radius, var(--control-radius-md));
    color: var(--text-primary);
    corner-shape: var(--cn-input-corner-shape, var(--control-corner-shape));
    transition: border-color 150ms, background 150ms, corner-shape 180ms;
}
.cn-input:hover {
    border-color: var(--border-hover);
    background: var(--input-bg-hover);
    corner-shape: var(--cn-input-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-input:focus {
    border-color: var(--border-focus);
    background: var(--input-bg-focus);
    corner-shape: var(--cn-input-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-input--error {
    border-color: var(--border-error);
}

.cn-input--sm { font-size: var(--cn-input-font-size-sm, var(--type-body-sm)); }
.cn-input--md { font-size: var(--cn-input-font-size-md, var(--type-body-md)); }
.cn-input--lg { font-size: var(--cn-input-font-size-lg, var(--type-body-lg)); }

/* ============================================================================
   Textarea
   ============================================================================ */

.cn-textarea {
    background: var(--cn-textarea-bg, var(--input-bg));
    border: 1px solid var(--cn-textarea-border, var(--border));
    border-radius: var(--cn-textarea-radius, var(--control-radius-md));
    color: var(--text-primary);
    corner-shape: var(--cn-textarea-corner-shape, var(--control-corner-shape));
    transition: border-color 150ms, background 150ms, corner-shape 180ms;
}
.cn-textarea:hover {
    border-color: var(--border-hover);
    background: var(--input-bg-hover);
    corner-shape: var(--cn-textarea-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-textarea:focus {
    border-color: var(--border-focus);
    background: var(--input-bg-focus);
    corner-shape: var(--cn-textarea-corner-shape-hover, var(--control-corner-shape-hover));
}

/* ============================================================================
   Label
   ============================================================================ */

.cn-label {
    color: var(--cn-label-color, var(--text-primary));
}
.cn-label--disabled {
    color: var(--text-tertiary);
}

/* ============================================================================
   Kbd
   ============================================================================ */

.cn-kbd {
    background: var(--cn-kbd-bg, var(--surface));
    border-color: var(--cn-kbd-border, var(--border));
    border-radius: var(--cn-kbd-radius, var(--compact-kbd-radius));
    color: var(--text-secondary);
    corner-shape: var(--cn-kbd-corner-shape, 1.2);
}

/* ============================================================================
   Checkbox
   ============================================================================ */

.cn-checkbox {
    border: 2px solid var(--cn-checkbox-border, var(--border));
    border-radius: var(--cn-checkbox-radius, var(--control-radius-sm));
    background: var(--cn-checkbox-bg, var(--input-bg));
    cursor: pointer;
    transition: background 150ms, border-color 150ms, transform 100ms;
    corner-shape: var(--cn-checkbox-corner-shape, var(--control-corner-shape));
}
.cn-checkbox:hover {
    border-color: var(--border-hover);
    transform: scale(1.05, 1.05);
}
.cn-checkbox--checked {
    background: var(--cn-checkbox-checked-bg, var(--primary));
    border-color: var(--cn-checkbox-checked-border, var(--primary));
}
.cn-checkbox--disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* ============================================================================
   Switch
   ============================================================================ */

.cn-switch {
    border-radius: 9999px;
    cursor: pointer;
    transition: background 200ms;
}
.cn-switch-track {
    background: var(--cn-switch-off-bg, var(--border));
    border-radius: 9999px;
}
.cn-switch-track--on {
    background: var(--cn-switch-on-bg, var(--primary));
}
.cn-switch-thumb {
    background: var(--cn-switch-thumb, var(--text-inverse));
    border-radius: 9999px;
}
.cn-switch--disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* ============================================================================
   Radio
   ============================================================================ */

.cn-radio {
    border: 2px solid var(--cn-radio-border, var(--border-secondary));
    border-radius: 9999px;
    cursor: pointer;
    transition: border-color 150ms, transform 100ms;
}
.cn-radio:hover {
    border-color: var(--cn-radio-hover-border, var(--primary));
    transform: scale(1.05, 1.05);
}
.cn-radio--selected {
    border-color: var(--cn-radio-selected, var(--primary));
}
.cn-radio-dot {
    background: var(--cn-radio-dot, var(--primary));
    border-radius: 9999px;
}
.cn-radio--disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* ============================================================================
   Tabs
   ============================================================================ */

.cn-tabs-list {
    background: var(--cn-tabs-list-bg, var(--surface-elevated));
    border-radius: var(--cn-tabs-list-radius, var(--container-radius));
    padding: var(--cn-tabs-list-padding, var(--overlay-gap));
    gap: var(--cn-tabs-list-gap, var(--overlay-gap));
    corner-shape: var(--cn-tabs-list-corner-shape, var(--container-corner-shape));
}
.cn-tabs-trigger {
    border-radius: var(--cn-tabs-trigger-radius, var(--control-radius-md));
    cursor: pointer;
    color: var(--text-secondary);
    corner-shape: var(--cn-tabs-trigger-corner-shape, var(--control-corner-shape));
    transition: background 150ms, color 150ms, corner-shape 180ms;
}
.cn-tabs-trigger:hover {
    color: var(--text-primary);
    background: var(--surface-overlay);
    corner-shape: var(--cn-tabs-trigger-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-tabs-trigger--active {
    background: var(--cn-tabs-active-bg, var(--background));
    color: var(--text-primary);
    box-shadow: theme(shadow-sm);
}
.cn-tabs-trigger--disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.cn-tabs-trigger--sm { height: var(--control-height-sm); padding: var(--control-py-sm) var(--control-px-sm); font-size: var(--type-action-sm); }
.cn-tabs-trigger--md { height: var(--control-height-md); padding: var(--control-py-md) var(--control-px-md); font-size: var(--type-action-md); }
.cn-tabs-trigger--lg { height: var(--control-height-lg); padding: var(--control-py-lg) var(--control-px-lg); font-size: var(--type-action-lg); }

/* ============================================================================
   Select
   ============================================================================ */

.cn-select-trigger {
    background: var(--cn-select-bg, var(--input-bg));
    border: 1px solid var(--cn-select-border, var(--border));
    border-radius: var(--cn-select-trigger-radius, var(--control-radius-md));
    cursor: pointer;
    color: var(--text-primary);
    corner-shape: var(--cn-select-trigger-corner-shape, var(--control-corner-shape));
    transition: border-color 150ms, background 150ms, corner-shape 180ms;
}
.cn-select-trigger:hover {
    border-color: var(--border-hover);
    background: var(--input-bg-hover);
    corner-shape: var(--cn-select-trigger-corner-shape-hover, var(--control-corner-shape-hover));
}

.cn-select-content {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-select-content-radius, var(--overlay-radius));
    padding: var(--cn-select-content-py, var(--overlay-gap));
    box-shadow: var(--cn-select-shadow, var(--overlay-shadow));
    corner-shape: var(--cn-select-content-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}

.cn-select-item {
    padding: var(--cn-select-item-py, var(--overlay-item-py)) var(--cn-select-item-px, var(--overlay-item-px));
    cursor: pointer;
    color: var(--text-primary);
    border-radius: var(--cn-select-item-radius, var(--control-radius-sm));
    corner-shape: var(--cn-select-item-corner-shape, var(--control-corner-shape));
    transition: background 100ms, corner-shape 180ms;
}
.cn-select-item:hover {
    background: var(--surface-elevated);
    corner-shape: var(--cn-select-item-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-select-item--selected {
    background: var(--surface-elevated);
}
.cn-select-value { max-width: 100%; }

/* ============================================================================
   Slider
   ============================================================================ */

.cn-slider-track {
    background: var(--cn-slider-track-bg, var(--surface-elevated));
    border-radius: 9999px;
}
.cn-slider-fill {
    background: var(--cn-slider-fill-bg, var(--primary));
    border-radius: 9999px;
}
.cn-slider-thumb {
    border: 2px solid var(--cn-slider-thumb-border, var(--border));
    border-radius: 9999px;
    background: var(--cn-slider-thumb-bg, var(--surface));
    cursor: pointer;
}

/* ============================================================================
   Progress
   ============================================================================ */

.cn-progress {
    background: var(--cn-progress-track, var(--accent-subtle));
    border-radius: 9999px;
    overflow: hidden;
}
.cn-progress-bar {
    background: var(--cn-progress-bar, var(--primary));
    border-radius: 9999px;
    transition: width 300ms;
}
.cn-progress--sm { height: var(--cn-progress-height-sm, var(--compact-progress-height-sm)); }
.cn-progress--md { height: var(--cn-progress-height-md, var(--compact-progress-height-md)); }
.cn-progress--lg { height: var(--cn-progress-height-lg, var(--compact-progress-height-lg)); }

/* ============================================================================
   Avatar
   ============================================================================ */

.cn-avatar {
    background: var(--cn-avatar-bg, var(--surface-elevated));
    border: 1px solid var(--cn-avatar-border, var(--border));
    border-radius: 9999px;
    overflow: hidden;
}
.cn-avatar--square {
    border-radius: var(--cn-avatar-square-radius, var(--control-radius-md));
}

/* ============================================================================
   Spinner
   ============================================================================ */

.cn-spinner {
    color: var(--cn-spinner-color, var(--primary));
}

/* ============================================================================
   Tooltip
   ============================================================================ */

.cn-tooltip {
    background: var(--cn-tooltip-bg, var(--tooltip-bg));
    color: var(--cn-tooltip-text, var(--tooltip-text));
    border-radius: var(--cn-tooltip-radius, var(--control-radius-sm));
    font-size: var(--cn-tooltip-font-size, var(--type-helper));
    padding: var(--cn-tooltip-py, var(--overlay-item-py)) var(--cn-tooltip-px, var(--overlay-item-px));
    corner-shape: var(--cn-tooltip-corner-shape, var(--control-corner-shape));
}

/* ============================================================================
   Dialog
   ============================================================================ */

.cn-dialog {
    background: var(--cn-dialog-bg, var(--surface));
    border: 1px solid var(--cn-dialog-border, var(--border));
    border-radius: var(--cn-dialog-radius, var(--container-radius));
    padding: var(--cn-dialog-padding, var(--container-padding));
    gap: var(--cn-dialog-gap, var(--container-section-gap));
    corner-shape: var(--cn-dialog-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}

/* ============================================================================
   Drawer
   ============================================================================ */

.cn-drawer {
    background: var(--cn-drawer-bg, var(--surface));
    border: 1px solid var(--cn-drawer-border, var(--border));
    corner-shape: var(--cn-drawer-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}
.cn-drawer-header {
    border-bottom: 1px solid var(--border);
    padding: var(--cn-drawer-header-padding, var(--container-padding-compact));
}
.cn-drawer-footer {
    padding: var(--cn-drawer-footer-padding, var(--container-padding-compact));
}

/* ============================================================================
   Sheet
   ============================================================================ */

.cn-sheet {
    background: var(--cn-sheet-bg, var(--surface));
    border: 1px solid var(--cn-sheet-border, var(--border));
    corner-shape: var(--cn-sheet-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}

/* ============================================================================
   Toast
   ============================================================================ */

.cn-toast {
    background: var(--cn-toast-bg, var(--surface));
    border: 1px solid var(--cn-toast-border, var(--border));
    border-radius: var(--cn-toast-radius, var(--container-radius));
    color: var(--text-primary);
    corner-shape: var(--cn-toast-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}
.cn-toast--success {
    border-left: 4px solid var(--success);
}
.cn-toast--warning {
    border-left: 4px solid var(--warning);
}
.cn-toast--error {
    border-left: 4px solid var(--error);
}
.cn-toast--info {
    border-left: 4px solid var(--info);
}

/* ============================================================================
   Accordion
   ============================================================================ */

.cn-accordion {
    background: var(--cn-accordion-bg, var(--surface-elevated));
    border: 1.5px solid var(--cn-accordion-border, var(--border));
    border-radius: var(--cn-accordion-radius, var(--container-radius));
    corner-shape: var(--cn-accordion-corner-shape, var(--container-corner-shape));
}
.cn-accordion-trigger {
    padding: var(--cn-accordion-trigger-py, var(--overlay-item-py)) var(--cn-accordion-trigger-px, var(--overlay-item-px));
    cursor: pointer;
    color: var(--text-primary);
    font-size: var(--cn-accordion-trigger-font-size, var(--type-action-md));
    text-overflow: ellipsis;
}
.cn-accordion-trigger:hover {
    background: var(--surface-overlay);
}
.cn-accordion-content {
    background: var(--cn-accordion-content-bg, var(--surface));
    border-top: 1px solid var(--border);
    color: var(--text-secondary);
}

/* ============================================================================
   Breadcrumb
   ============================================================================ */

.cn-breadcrumb {
    gap: var(--cn-breadcrumb-gap, var(--compact-cluster-gap-md));
    color: var(--text-secondary);
}
.cn-breadcrumb-item {
    color: var(--text-secondary);
    cursor: pointer;
    text-decoration: underline;
    text-decoration-color: transparent;
    text-decoration-thickness: 1.5px;
}
.cn-breadcrumb-item:hover {
    color: var(--text-primary);
    text-decoration-color: var(--primary);
}
.cn-breadcrumb-item--active {
    color: var(--text-primary);
    text-decoration-color: var(--primary);
}
.cn-breadcrumb-label { max-width: 100%; }

/* ============================================================================
   Pagination
   ============================================================================ */

.cn-pagination {
    gap: var(--cn-pagination-gap, var(--compact-cluster-gap-sm));
}
.cn-pagination-btn {
    border: 1px solid var(--border);
    border-radius: var(--cn-pagination-radius, var(--control-radius-md));
    cursor: pointer;
    color: var(--text-primary);
    corner-shape: var(--cn-pagination-corner-shape, var(--control-corner-shape));
    transition: background 150ms, corner-shape 180ms;
}
.cn-pagination-btn:hover {
    background: var(--surface-elevated);
    corner-shape: var(--cn-pagination-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-pagination-btn--active {
    background: var(--primary);
    color: var(--text-inverse);
    border-color: var(--primary);
}
.cn-pagination-btn--disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* ============================================================================
   Navigation Menu
   ============================================================================ */

.cn-nav-menu {
    gap: var(--cn-nav-menu-gap, var(--compact-cluster-gap-sm));
}
.cn-nav-link {
    padding: var(--cn-nav-link-py, var(--overlay-item-py)) var(--cn-nav-link-px, var(--overlay-item-px));
    border-radius: var(--cn-nav-link-radius, var(--control-radius-md));
    cursor: pointer;
    color: var(--text-secondary);
    corner-shape: var(--cn-nav-link-corner-shape, var(--control-corner-shape));
    transition: background 150ms, color 150ms, corner-shape 180ms;
}
.cn-nav-link:hover {
    background: var(--surface-elevated);
    color: var(--text-primary);
    corner-shape: var(--cn-nav-link-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-nav-link--active {
    background: var(--surface-elevated);
    color: var(--text-primary);
    text-decoration: underline;
    text-decoration-color: var(--primary);
    text-decoration-thickness: 1.5px;
}
.cn-nav-link__label { max-width: 100%; }

/* ============================================================================
   Sidebar
   ============================================================================ */

.cn-sidebar {
    background: var(--cn-sidebar-bg, var(--surface));
    border-right: 1px solid var(--border);
}
.cn-sidebar-item {
    padding: var(--cn-sidebar-item-py, var(--overlay-item-py)) var(--cn-sidebar-item-px, var(--overlay-item-px));
    border-radius: var(--cn-sidebar-item-radius, var(--control-radius-md));
    cursor: pointer;
    color: var(--text-secondary);
    corner-shape: var(--cn-sidebar-item-corner-shape, var(--control-corner-shape));
    transition: background 150ms, color 150ms, corner-shape 180ms;
}
.cn-sidebar-item:hover {
    background: var(--surface-elevated);
    color: var(--text-primary);
    corner-shape: var(--cn-sidebar-item-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-sidebar-item--active {
    background: var(--primary);
    color: var(--text-inverse);
}
.cn-sidebar-item__label { max-width: 100%; }

/* ============================================================================
   Scroll Area
   ============================================================================ */

.cn-scroll-area {
    overflow: hidden;
}

/* ============================================================================
   Dropdown Menu
   ============================================================================ */

.cn-dropdown-menu {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-dropdown-radius, var(--overlay-radius));
    padding: var(--cn-dropdown-padding, var(--overlay-gap));
    box-shadow: var(--cn-dropdown-shadow, var(--overlay-shadow));
    corner-shape: var(--cn-dropdown-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}
.cn-dropdown-item {
    padding: var(--cn-dropdown-item-py, var(--overlay-item-py)) var(--cn-dropdown-item-px, var(--overlay-item-px));
    border-radius: var(--cn-dropdown-item-radius, var(--control-radius-sm));
    cursor: pointer;
    color: var(--text-primary);
    font-size: var(--cn-dropdown-font-size, var(--type-body-md));
    corner-shape: var(--cn-dropdown-item-corner-shape, var(--control-corner-shape));
    transition: background 100ms, corner-shape 180ms;
}
.cn-dropdown-item:hover {
    background: var(--surface-elevated);
    corner-shape: var(--cn-dropdown-item-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-dropdown-item--disabled {
    opacity: 0.5;
    cursor: not-allowed;
}
.cn-dropdown-item--destructive {
    color: var(--error);
}
.cn-dropdown-item__label { max-width: 100%; }
.cn-menu-shortcut {
    text-decoration: underline;
    text-decoration-color: var(--text-tertiary);
    text-decoration-thickness: 1px;
}

/* ============================================================================
   Context Menu
   ============================================================================ */

.cn-context-menu {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-context-menu-radius, var(--overlay-radius));
    padding: var(--cn-context-menu-padding, var(--overlay-gap));
    box-shadow: var(--cn-context-menu-shadow, var(--overlay-shadow));
    corner-shape: var(--cn-context-menu-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}
.cn-context-menu-item {
    padding: var(--cn-context-menu-item-py, var(--overlay-item-py)) var(--cn-context-menu-item-px, var(--overlay-item-px));
    border-radius: var(--cn-context-menu-item-radius, var(--control-radius-sm));
    cursor: pointer;
    color: var(--text-primary);
    font-size: var(--cn-context-menu-font-size, var(--type-body-md));
    corner-shape: var(--cn-context-menu-item-corner-shape, var(--control-corner-shape));
    transition: background 100ms, corner-shape 180ms;
}
.cn-context-menu-item:hover {
    background: var(--surface-elevated);
    corner-shape: var(--cn-context-menu-item-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-context-menu-item__label { max-width: 100%; }

/* ============================================================================
   Menubar
   ============================================================================ */

.cn-menubar {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-menubar-radius, var(--overlay-radius));
    padding: var(--cn-menubar-padding, var(--overlay-gap));
    gap: var(--cn-menubar-gap, var(--overlay-gap));
    corner-shape: var(--cn-menubar-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}
.cn-menubar-trigger {
    padding: var(--cn-menubar-trigger-py, var(--overlay-item-py)) var(--cn-menubar-trigger-px, var(--overlay-item-px));
    border-radius: var(--cn-menubar-trigger-radius, var(--control-radius-sm));
    cursor: pointer;
    color: var(--text-primary);
    font-size: var(--cn-menubar-font-size, var(--type-action-md));
    corner-shape: var(--cn-menubar-trigger-corner-shape, var(--control-corner-shape));
    transition: background 100ms, corner-shape 180ms;
}
.cn-menubar-trigger:hover {
    background: var(--surface-elevated);
    corner-shape: var(--cn-menubar-trigger-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-menubar-trigger__label { max-width: 100%; }

/* ============================================================================
   Popover
   ============================================================================ */

.cn-popover-content {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-popover-radius, var(--overlay-radius));
    padding: var(--cn-popover-padding, var(--container-padding-compact));
    box-shadow: var(--cn-popover-shadow, var(--overlay-shadow));
    corner-shape: var(--cn-popover-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}

/* ============================================================================
   Hover Card
   ============================================================================ */

.cn-hover-card-content {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-hover-card-radius, var(--overlay-radius));
    padding: var(--cn-hover-card-padding, var(--container-padding-compact));
    box-shadow: var(--cn-hover-card-shadow, var(--overlay-shadow));
    corner-shape: var(--cn-hover-card-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}

/* ============================================================================
   Tree View
   ============================================================================ */

.cn-tree-node {
    padding: var(--cn-tree-node-py, var(--overlay-gap)) var(--cn-tree-node-px, var(--control-px-sm));
    border-radius: var(--cn-tree-node-radius, var(--control-radius-sm));
    cursor: pointer;
    color: var(--text-primary);
    corner-shape: var(--cn-tree-node-corner-shape, var(--control-corner-shape));
    transition: background 100ms, corner-shape 180ms;
}
.cn-tree-node:hover {
    background: var(--surface-elevated);
    corner-shape: var(--cn-tree-node-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-tree-node--selected {
    background: var(--primary);
    color: var(--text-inverse);
}

/* ============================================================================
   Resizable
   ============================================================================ */

.cn-resizable-handle {
    background: var(--border);
    transition: background 150ms;
}
.cn-resizable-handle:hover {
    background: var(--primary);
}

/* ============================================================================
   Collapsible
   ============================================================================ */

.cn-collapsible-trigger {
    cursor: pointer;
    color: var(--text-primary);
}

/* ============================================================================
   Combobox
   ============================================================================ */

.cn-combobox-trigger {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-combobox-trigger-radius, var(--control-radius-md));
    cursor: pointer;
    color: var(--text-primary);
    corner-shape: var(--cn-combobox-trigger-corner-shape, var(--control-corner-shape));
    transition: border-color 150ms, background 150ms, corner-shape 180ms;
}
.cn-combobox-trigger:hover {
    border-color: var(--border-hover);
    corner-shape: var(--cn-combobox-trigger-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-combobox-content {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--cn-combobox-content-radius, var(--overlay-radius));
    padding: var(--cn-combobox-content-py, var(--overlay-gap));
    box-shadow: var(--cn-combobox-shadow, var(--overlay-shadow));
    corner-shape: var(--cn-combobox-content-corner-shape, var(--overlay-corner-shape));
    backdrop-filter: glass;
}
.cn-combobox-item {
    padding: var(--cn-combobox-item-py, var(--overlay-item-py)) var(--cn-combobox-item-px, var(--overlay-item-px));
    cursor: pointer;
    color: var(--text-primary);
    border-radius: var(--cn-combobox-item-radius, var(--control-radius-sm));
    corner-shape: var(--cn-combobox-item-corner-shape, var(--control-corner-shape));
    transition: background 100ms, corner-shape 180ms;
}
.cn-combobox-item:hover {
    background: var(--surface-elevated);
    corner-shape: var(--cn-combobox-item-corner-shape-hover, var(--control-corner-shape-hover));
}
.cn-combobox-value { max-width: 100%; }
.cn-combobox-item__label { max-width: 100%; }
"#;
