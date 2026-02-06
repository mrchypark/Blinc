# Changelog

All notable changes to `blinc_layout` will be documented in this file.

## [Unreleased]

### Added
- `PINCH` event support in `EventContext` (center and scale fields)

## [0.1.12] - 2025-01-19

### Added
- `apply_touch_scroll_delta()` method for touch velocity tracking
- `scroll_time` field in `EventContext` for momentum scrolling
- `dispatch_scroll_chain_with_time()` in RenderTree for mobile scroll dispatch
- Momentum deceleration in scroll physics tick for touch devices

### Changed
- Scroll physics now supports velocity-based momentum scrolling
- `on_scroll_end()` starts momentum if velocity exceeds threshold

## [0.1.1] - Initial Release

- Initial public release with layout engine and scroll widgets
