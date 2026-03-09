# blinc_cn Latest CSS Defaults Design

## Goal

Apply recently added CSS features to `blinc_cn` defaults aggressively enough that the default component set visibly benefits from the newer engine capabilities, while preserving sensible out-of-the-box UI behavior.

## Scope

- Prioritize the default `blinc_cn` component set over `blinc_layout` primitives.
- Keep the token-first structure introduced in `blinc_theme`.
- Use modern CSS features in defaults, not only in demos:
  - `corner-shape`
  - `backdrop-filter`
  - `text-overflow`
  - `text-decoration`
  - `pointer-events`
  - `cursor`
  - `mix-blend-mode` where it is decorative and low-risk

## Design

### 1. Keep semantic tokens as the backbone

Recent CSS usage should still route through semantic component defaults where practical. New CSS-heavy defaults should consume `control`, `container`, and `overlay` semantics instead of reintroducing component-local hardcoded geometry.

### 2. Apply expressive CSS by component tier

- Controls (`Button`, `Input`, `Textarea`, `Tabs`, `Select`, `Combobox`)
  - Add `corner-shape` defaults and transitions.
  - Apply truncation classes to labels and selected values.
  - Use richer interactive state styling for link/button-like content.
- Floating surfaces (`Dropdown`, `ContextMenu`, `Menubar`, `Popover`, `HoverCard`, `Dialog`, `Toast`)
  - Add `backdrop-filter` defaults.
  - Add `corner-shape` for a more modern silhouette.
- Navigation/content labels (`NavigationMenu`, `Sidebar`, `Breadcrumb`, `Menubar`)
  - Add truncation and text-decoration defaults where appropriate.
- Decorative/supporting surfaces (`Skeleton`, `Badge`, `Kbd`)
  - Use low-risk modern CSS such as subtle `mix-blend-mode`, `corner-shape`, and interaction utilities.

### 3. Introduce reusable utility classes in `cn_styles`

The stylesheet should gain a small set of reusable CSS utilities so components can opt into recent features without duplicating declarations:

- `.cn-truncate`
- `.cn-decorative`
- `.cn-link-text`
- shared overlay material rules

### 4. Update component markup where CSS needs a target

Some newer CSS properties only matter if text or decorative children expose a class target. Add classes to button labels, select values, combobox values, nav labels, sidebar labels, breadcrumb labels, and menu shortcuts/icons where needed.

## Success Criteria

- The PR no longer relies only on `var()` and pseudo-class support; it uses recently added CSS features in the shipped defaults.
- Default `blinc_cn` controls and overlays visibly use `corner-shape` and `backdrop-filter`.
- Common long labels truncate gracefully in buttons, selects, nav, sidebar, and breadcrumb components.
- Link-like defaults use CSS-driven text decoration rather than only color changes.
