# blinc_cn Components Needed for blinc_debugger

## Summary

Based on the debugger UI design and the current blinc_cn component inventory, the following components are **required** or **recommended** for completing the debugger implementation.

## Required Components (Not Yet Implemented)

### 1. Icon System (High Priority)

**Status**: Planned in blinc-cn-components.md
**Used in**: Sidebar, toolbar buttons, tree nodes, timeline markers

```rust
// Usage examples from debugger:
cn::icon("play")        // Timeline play button
cn::icon("pause")       // Timeline pause button
cn::icon("chevron-right") // Tree expand
cn::icon("search")      // Search bar
cn::icon("settings")    // Sidebar
```

**Blocking**: Many UI elements currently use Unicode placeholder characters.

### 2. Tree View (Medium Priority)

**Status**: Not in plan - needs to be added
**Used in**: Tree Panel (element tree visualization)

```rust
cn::tree()
    .node("root", |n| n
        .label("Root")
        .expanded(true)
        .children([
            cn::tree_node("child1").label("Header"),
            cn::tree_node("child2").label("Main").children([...]),
        ])
    )
    .on_select(|id| { ... })
```

**Note**: This is a complex component requiring:
- Collapsible nodes with animation
- Selection state
- Keyboard navigation
- Virtual scrolling for large trees

### 3. Charts (Canvas-based) (Low Priority)

**Status**: Planned (Data Display category)
**Used in**: Preview panel stats, event distribution

Components needed:
- `cn::line_chart()` - Time series data
- `cn::bar_chart()` - Event type distribution

**Workaround**: Placeholder divs for now. Can implement with `canvas()` primitive directly if needed.

## Already Available (Can Use)

These components from blinc_cn are already implemented and can be used:

| Component | Use Case in Debugger |
|-----------|---------------------|
| **Button** | Toolbar buttons, control buttons |
| **Input** | Search bar, filter inputs |
| **Select** | Dropdown filters |
| **Tabs** | Panel tabs (if needed) |
| **Slider** | Timeline scrubber |
| **Progress** | Loading indicators |
| **Badge** | Event count badges |
| **Card** | Panel cards |
| **Separator** | Panel dividers |
| **Tooltip** | Button tooltips |
| **Dialog** | File open dialog, settings |
| **Kbd** | Keyboard shortcuts display |
| **Accordion** | Inspector sections |
| **Collapsible** | Tree node expansion |
| **Sidebar** | Left navigation |
| **Toast** | Notifications |

## Recommended Additions to blinc_cn Plan

### 1. Tree View Component

Add to **Layout** category:

```markdown
| Component | Primitives Used | Status |
|-----------|-----------------|--------|
| **Tree** | div, Collapsible, scroll | Planned |
```

### 2. Scrubber Component

Specialized slider for timeline scrubbing:

```markdown
| Component | Primitives Used | Status |
|-----------|-----------------|--------|
| **Scrubber** | div, drag, AnimatedValue | Planned |
```

## Implementation Priority

1. **Icon System** - Blocks visual polish across all panels
2. **Tree View** - Blocks Tree Panel functionality
3. **Charts** - Low priority, can use canvas directly

## Documentation

For API reference, always consult:

- `docs/book/` - Blinc documentation
- `blinc_cn` crate - Component API examples and patterns
