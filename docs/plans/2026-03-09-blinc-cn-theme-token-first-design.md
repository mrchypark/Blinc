# Blinc CN Theme-Token-First Defaults Design

## Goal

Make `blinc_cn` visually coherent by moving default sizing, spacing, typography-role, and overlay/container decisions into `blinc_theme` first, then making `blinc_cn` consume those semantic tokens instead of local constants.

## Problem

`blinc_cn` currently mixes theme tokens with component-local constants. Colors mostly come from tokens, but radii, paddings, gaps, heights, and font sizes are often repeated directly in component code and `cn_styles.rs`. This weakens first-run coherence and makes shadcn-like presets feel only partially centralized.

`blinc_layout` should remain the primitive/widget substrate. Polished defaults belong in `blinc_cn`.

## Design

### 1. Keep primitive tokens, add semantic component roles

Preserve the current primitive scales in `blinc_theme`:

- color tokens
- spacing scale
- radius scale
- typography scale
- shadow scale

Add semantic component-token groups on top of those primitives:

- control metrics
  - heights for `sm/md/lg`
  - horizontal/vertical paddings for `sm/md/lg`
  - default control radius
- container metrics
  - card/dialog/drawer/content padding
  - section/header/footer gaps
  - default container radius
- overlay metrics
  - menu/select/popover/content padding
  - overlay radius
  - overlay shadow
- typography roles
  - action text
  - body text
  - helper text
  - label text
  - title text
- compact badges/kbd/chips
  - badge font size
  - badge paddings
  - chip radius

The semantic layer must be derived from primitive scales in defaults and presets instead of inventing independent numbers everywhere.

### 2. Make `Theme` and `ThemeState` expose semantic component tokens

Introduce a dedicated semantic token struct in `blinc_theme`, likely alongside `ColorTokens`, `SpacingTokens`, `RadiusTokens`, and `TypographyTokens`.

`Theme` should expose the semantic token set directly, and `ThemeState` should cache and return it just like the other token families. This keeps consumers from rebuilding ad-hoc mappings in every component.

### 3. Rewire `blinc_cn` to consume semantic tokens first

Replace repeated hard-coded defaults in:

- `cn_styles.rs`
- button/input/textarea size logic
- card/alert/badge surface defaults
- dropdown/menu/context-menu/menubar/select overlays
- tabs/sidebar/avatar/progress where visual constants still leak

Preferred lookup order:

1. explicit component builder override
2. semantic component token
3. primitive token only when the semantic token is intentionally derived from it

Avoid direct numeric literals unless they are geometric invariants such as fully-rounded pills or track math.

### 4. Keep CSS overridable, but token-backed by default

`cn_styles.rs` should continue to define CSS variables and class defaults, but its fallback values should mostly point at semantic CSS variables exported from `ThemeState::to_css_variable_map()`, rather than raw literals like `12px`, `16px`, or `14px`.

### 5. Documentation boundary

Document `blinc_layout` as a primitive/widget layer with sane behavioral defaults.
Document `blinc_cn` as the polished design-system layer responsible for default visual quality.

## Non-Goals

- Reworking all platform-native themes
- Redesigning the color palettes
- Turning `blinc_layout` into a shadcn-style component library

## Success Criteria

- `blinc_cn` components in the same preset share a stable radius/spacing/type system by default.
- `cn_styles.rs` loses most visual hard-coded literals for common component surfaces and controls.
- overlay components share one default overlay token family.
- control-like components share one default control token family.
- docs clearly separate primitive defaults from polished component defaults.
