# Card Text Wrap Design

**Problem**

Wrapping text inside `blinc_cn::Card` can still measure as a single line because the text node often reaches Taffy's measure callback without a definite width. The immediate trigger is cross-axis `items_start()` on block-oriented containers, which allows text children to stay shrink-wrapped instead of stretching to the container width.

**Confirmed Root Cause**

- `blinc_layout::text::Text` uses `width = Auto` and `max_width = 100%` when wrapping is enabled.
- The actual wrap width comes from `available_space.width` during `text_measure_function`.
- If Taffy reports `AvailableSpace::MaxContent`, measurement falls back to `max_width = None`, which produces single-line metrics.
- `blinc_cn::Card` and `blinc_cn::CardHeader` currently default to `items_start()`, so block text children are easy to route into that intrinsic-width path.

**Chosen Approach**

1. Change block-oriented card containers to `items_stretch()` by default.
2. Add explicit width helpers to `Text` so callers can state paragraph intent directly.
3. Add debug diagnostics when wrapping text is measured without a definite width, and use fixed-width style hints when available.

**Why This Approach**

- It fixes the default card case without requiring wrapper `div()` workarounds.
- It gives app code a stable API for paragraph-like text outside cards.
- It keeps the change surgical: no renderer rewrite, no speculative layout abstraction.

**Compatibility**

- Some existing card children may become wider because they now inherit stretch behavior.
- The escape hatch remains explicit: callers can opt back into natural-width behavior with `items_start()` or `align_self_start()`.

**Acceptance Criteria**

- `card().w(480.0).child(text(long_text))` wraps inside the card width.
- `card_header().description(long_text)` wraps inside the header width.
- `Text` exposes paragraph-friendly width helpers (`w_full`, `max_w`) that survive later builder mutations.
- Wrap regressions are covered in both `blinc_layout` and `blinc_cn`.
