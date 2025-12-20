# Blinc Design & Motion Tokens

This document outlines the design token system and motion presets for the Blinc UI Framework.

## Overview

Design tokens are the atomic values that define the visual language of your application. Motion tokens define how elements animate and transition. Together, they create a consistent, cohesive user experience.

**Key Feature: DSL Extensibility** - All tokens are fully extensible from the Blinc DSL, allowing custom theming and UX behaviors without modifying core framework code.

## 1. Color Tokens

### Base Colors

```rust
// Color palette definition
pub mod tokens {
    pub mod color {
        // Neutrals (Gray scale)
        pub const GRAY_50: Color = Color::hex(0xFAFAFA);
        pub const GRAY_100: Color = Color::hex(0xF5F5F5);
        pub const GRAY_200: Color = Color::hex(0xEEEEEE);
        pub const GRAY_300: Color = Color::hex(0xE0E0E0);
        pub const GRAY_400: Color = Color::hex(0xBDBDBD);
        pub const GRAY_500: Color = Color::hex(0x9E9E9E);
        pub const GRAY_600: Color = Color::hex(0x757575);
        pub const GRAY_700: Color = Color::hex(0x616161);
        pub const GRAY_800: Color = Color::hex(0x424242);
        pub const GRAY_900: Color = Color::hex(0x212121);

        // Primary (Customizable per-theme)
        pub const PRIMARY_50: Color = Color::hex(0xE3F2FD);
        pub const PRIMARY_100: Color = Color::hex(0xBBDEFB);
        pub const PRIMARY_200: Color = Color::hex(0x90CAF9);
        pub const PRIMARY_300: Color = Color::hex(0x64B5F6);
        pub const PRIMARY_400: Color = Color::hex(0x42A5F5);
        pub const PRIMARY_500: Color = Color::hex(0x2196F3);  // Main primary
        pub const PRIMARY_600: Color = Color::hex(0x1E88E5);
        pub const PRIMARY_700: Color = Color::hex(0x1976D2);
        pub const PRIMARY_800: Color = Color::hex(0x1565C0);
        pub const PRIMARY_900: Color = Color::hex(0x0D47A1);

        // Semantic colors
        pub const SUCCESS: Color = Color::hex(0x4CAF50);
        pub const WARNING: Color = Color::hex(0xFF9800);
        pub const ERROR: Color = Color::hex(0xF44336);
        pub const INFO: Color = Color::hex(0x2196F3);
    }
}
```

### Semantic Color Roles

| Token | Light Mode | Dark Mode | Usage |
|-------|------------|-----------|-------|
| `background` | GRAY_50 | GRAY_900 | Page/app background |
| `surface` | WHITE | GRAY_800 | Card/panel surfaces |
| `on-surface` | GRAY_900 | GRAY_100 | Text on surfaces |
| `primary` | PRIMARY_500 | PRIMARY_300 | Primary actions, links |
| `on-primary` | WHITE | GRAY_900 | Text on primary |
| `outline` | GRAY_300 | GRAY_600 | Borders, dividers |
| `disabled` | GRAY_400 | GRAY_600 | Disabled elements |

## 2. Typography Tokens

### Font Scale

```rust
pub mod tokens {
    pub mod typography {
        // Size scale (rem-based for accessibility)
        pub const SIZE_XS: f32 = 12.0;
        pub const SIZE_SM: f32 = 14.0;
        pub const SIZE_MD: f32 = 16.0;  // Base
        pub const SIZE_LG: f32 = 18.0;
        pub const SIZE_XL: f32 = 20.0;
        pub const SIZE_2XL: f32 = 24.0;
        pub const SIZE_3XL: f32 = 30.0;
        pub const SIZE_4XL: f32 = 36.0;
        pub const SIZE_5XL: f32 = 48.0;
        pub const SIZE_6XL: f32 = 60.0;

        // Line heights
        pub const LINE_TIGHT: f32 = 1.25;
        pub const LINE_SNUG: f32 = 1.375;
        pub const LINE_NORMAL: f32 = 1.5;
        pub const LINE_RELAXED: f32 = 1.625;
        pub const LINE_LOOSE: f32 = 2.0;

        // Font weights
        pub const WEIGHT_THIN: u16 = 100;
        pub const WEIGHT_LIGHT: u16 = 300;
        pub const WEIGHT_REGULAR: u16 = 400;
        pub const WEIGHT_MEDIUM: u16 = 500;
        pub const WEIGHT_SEMIBOLD: u16 = 600;
        pub const WEIGHT_BOLD: u16 = 700;
        pub const WEIGHT_BLACK: u16 = 900;

        // Letter spacing
        pub const TRACKING_TIGHTER: f32 = -0.05;
        pub const TRACKING_TIGHT: f32 = -0.025;
        pub const TRACKING_NORMAL: f32 = 0.0;
        pub const TRACKING_WIDE: f32 = 0.025;
        pub const TRACKING_WIDER: f32 = 0.05;
    }
}
```

### Type Styles

| Style | Size | Weight | Line Height | Tracking |
|-------|------|--------|-------------|----------|
| `display-lg` | 6XL | BOLD | TIGHT | TIGHTER |
| `display-md` | 5XL | BOLD | TIGHT | TIGHTER |
| `display-sm` | 4XL | SEMIBOLD | TIGHT | TIGHT |
| `heading-lg` | 3XL | SEMIBOLD | SNUG | TIGHT |
| `heading-md` | 2XL | SEMIBOLD | SNUG | NORMAL |
| `heading-sm` | XL | SEMIBOLD | SNUG | NORMAL |
| `body-lg` | LG | REGULAR | RELAXED | NORMAL |
| `body-md` | MD | REGULAR | NORMAL | NORMAL |
| `body-sm` | SM | REGULAR | NORMAL | NORMAL |
| `label-lg` | MD | MEDIUM | TIGHT | WIDE |
| `label-md` | SM | MEDIUM | TIGHT | WIDE |
| `label-sm` | XS | MEDIUM | TIGHT | WIDER |

## 3. Spacing Tokens

### Spacing Scale

```rust
pub mod tokens {
    pub mod spacing {
        pub const SPACE_0: f32 = 0.0;
        pub const SPACE_PX: f32 = 1.0;
        pub const SPACE_0_5: f32 = 2.0;
        pub const SPACE_1: f32 = 4.0;
        pub const SPACE_1_5: f32 = 6.0;
        pub const SPACE_2: f32 = 8.0;
        pub const SPACE_2_5: f32 = 10.0;
        pub const SPACE_3: f32 = 12.0;
        pub const SPACE_3_5: f32 = 14.0;
        pub const SPACE_4: f32 = 16.0;
        pub const SPACE_5: f32 = 20.0;
        pub const SPACE_6: f32 = 24.0;
        pub const SPACE_7: f32 = 28.0;
        pub const SPACE_8: f32 = 32.0;
        pub const SPACE_9: f32 = 36.0;
        pub const SPACE_10: f32 = 40.0;
        pub const SPACE_11: f32 = 44.0;
        pub const SPACE_12: f32 = 48.0;
        pub const SPACE_14: f32 = 56.0;
        pub const SPACE_16: f32 = 64.0;
        pub const SPACE_20: f32 = 80.0;
        pub const SPACE_24: f32 = 96.0;
        pub const SPACE_28: f32 = 112.0;
        pub const SPACE_32: f32 = 128.0;
    }
}
```

## 4. Border & Shadow Tokens

### Border Radius

```rust
pub mod tokens {
    pub mod radius {
        pub const NONE: f32 = 0.0;
        pub const SM: f32 = 2.0;
        pub const DEFAULT: f32 = 4.0;
        pub const MD: f32 = 6.0;
        pub const LG: f32 = 8.0;
        pub const XL: f32 = 12.0;
        pub const XXL: f32 = 16.0;
        pub const XXXL: f32 = 24.0;
        pub const FULL: f32 = 9999.0;  // Pill shape
    }
}
```

### Shadows

```rust
pub mod tokens {
    pub mod shadow {
        // box-shadow: offset-x offset-y blur spread color
        pub const SM: Shadow = Shadow::new(0.0, 1.0, 2.0, 0.0, Color::rgba(0, 0, 0, 0.05));
        pub const DEFAULT: Shadow = Shadow::new(0.0, 1.0, 3.0, 0.0, Color::rgba(0, 0, 0, 0.1));
        pub const MD: Shadow = Shadow::new(0.0, 4.0, 6.0, -1.0, Color::rgba(0, 0, 0, 0.1));
        pub const LG: Shadow = Shadow::new(0.0, 10.0, 15.0, -3.0, Color::rgba(0, 0, 0, 0.1));
        pub const XL: Shadow = Shadow::new(0.0, 20.0, 25.0, -5.0, Color::rgba(0, 0, 0, 0.1));
        pub const XXL: Shadow = Shadow::new(0.0, 25.0, 50.0, -12.0, Color::rgba(0, 0, 0, 0.25));
        pub const INNER: Shadow = Shadow::inset(0.0, 2.0, 4.0, 0.0, Color::rgba(0, 0, 0, 0.05));
    }
}
```

## 5. Motion Tokens

Motion tokens define how elements animate and transition, creating a consistent feel.

### Duration Tokens

```rust
pub mod tokens {
    pub mod duration {
        pub const INSTANT: u32 = 0;      // No animation
        pub const FAST: u32 = 100;       // Quick micro-interactions
        pub const NORMAL: u32 = 200;     // Standard transitions
        pub const SLOW: u32 = 300;       // Deliberate animations
        pub const SLOWER: u32 = 500;     // Emphasis animations
        pub const SLOWEST: u32 = 1000;   // Complex sequences
    }
}
```

### Easing Tokens

```rust
pub mod tokens {
    pub mod easing {
        // Standard easings
        pub const LINEAR: Easing = Easing::Linear;

        // Ease out (decelerate) - most common
        pub const EASE_OUT: Easing = Easing::CubicBezier(0.0, 0.0, 0.2, 1.0);
        pub const EASE_OUT_QUAD: Easing = Easing::EaseOutQuad;
        pub const EASE_OUT_CUBIC: Easing = Easing::EaseOutCubic;
        pub const EASE_OUT_QUART: Easing = Easing::EaseOutQuart;
        pub const EASE_OUT_EXPO: Easing = Easing::CubicBezier(0.16, 1.0, 0.3, 1.0);

        // Ease in (accelerate) - for exits
        pub const EASE_IN: Easing = Easing::CubicBezier(0.4, 0.0, 1.0, 1.0);
        pub const EASE_IN_QUAD: Easing = Easing::EaseInQuad;
        pub const EASE_IN_CUBIC: Easing = Easing::EaseInCubic;

        // Ease in-out (symmetric) - for attention
        pub const EASE_IN_OUT: Easing = Easing::CubicBezier(0.4, 0.0, 0.2, 1.0);
        pub const EASE_IN_OUT_QUAD: Easing = Easing::EaseInOutQuad;
        pub const EASE_IN_OUT_CUBIC: Easing = Easing::EaseInOutCubic;

        // Special
        pub const BOUNCE: Easing = Easing::CubicBezier(0.34, 1.56, 0.64, 1.0);
        pub const ANTICIPATE: Easing = Easing::CubicBezier(0.36, 0.0, 0.66, -0.56);
    }
}
```

### Spring Presets

These are the physics-based spring presets that create natural motion:

```rust
pub mod tokens {
    pub mod spring {
        /// A gentle, slow spring (good for page transitions)
        /// - Low stiffness = slow movement
        /// - Moderate damping = smooth stop
        pub const GENTLE: SpringConfig = SpringConfig {
            stiffness: 120.0,
            damping: 14.0,
            mass: 1.0,
        };

        /// A wobbly spring with overshoot (good for playful UI)
        /// - Moderate stiffness = medium speed
        /// - Low damping = more oscillation
        pub const WOBBLY: SpringConfig = SpringConfig {
            stiffness: 180.0,
            damping: 12.0,
            mass: 1.0,
        };

        /// Default stiff spring (good for buttons, quick feedback)
        /// - High stiffness = fast movement
        /// - Balanced damping = minimal overshoot
        pub const STIFF: SpringConfig = SpringConfig {
            stiffness: 400.0,
            damping: 30.0,
            mass: 1.0,
        };

        /// Very snappy spring (good for toggles, instant feedback)
        /// - Very high stiffness = very fast
        /// - High damping = no overshoot
        pub const SNAPPY: SpringConfig = SpringConfig {
            stiffness: 600.0,
            damping: 40.0,
            mass: 1.0,
        };

        /// Slow, heavy spring (good for dramatic emphasis)
        /// - Low stiffness = slow
        /// - High damping relative to stiffness = critically damped
        pub const MOLASSES: SpringConfig = SpringConfig {
            stiffness: 100.0,
            damping: 20.0,
            mass: 1.0,
        };

        /// Bouncy spring (good for celebratory moments)
        /// - High stiffness = fast
        /// - Low damping = lots of bounce
        pub const BOUNCY: SpringConfig = SpringConfig {
            stiffness: 500.0,
            damping: 15.0,
            mass: 1.0,
        };

        /// Smooth spring for layout animations
        /// - Medium stiffness = smooth
        /// - Slightly overdamped = no overshoot
        pub const LAYOUT: SpringConfig = SpringConfig {
            stiffness: 300.0,
            damping: 35.0,
            mass: 1.0,
        };
    }
}
```

### Spring Damping Guide

Understanding spring damping:

- **Underdamped** (`damping < critical`): Oscillates before settling. More playful, natural feel.
- **Critically damped** (`damping = critical`): Fastest settling without overshoot. Very snappy.
- **Overdamped** (`damping > critical`): Slow settling without overshoot. Heavy, deliberate feel.

The critical damping value is: `critical = 2 * sqrt(stiffness * mass)`

## 6. Widget State Tokens

### Opacity Tokens for States

```rust
pub mod tokens {
    pub mod opacity {
        pub const DISABLED: f32 = 0.38;     // Disabled elements
        pub const HINT: f32 = 0.60;         // Secondary text, hints
        pub const MEDIUM: f32 = 0.75;       // Medium emphasis
        pub const HIGH: f32 = 0.87;         // High emphasis text
        pub const FULL: f32 = 1.0;          // Full opacity

        // State overlays (applied on hover/press/focus)
        pub const HOVER_OVERLAY: f32 = 0.04;
        pub const FOCUS_OVERLAY: f32 = 0.12;
        pub const PRESS_OVERLAY: f32 = 0.12;
        pub const DRAG_OVERLAY: f32 = 0.16;
    }
}
```

### Scale Tokens for Interactions

```rust
pub mod tokens {
    pub mod scale {
        pub const PRESSED: f32 = 0.97;      // Slight scale down on press
        pub const HOVERED: f32 = 1.02;      // Slight scale up on hover
        pub const SELECTED: f32 = 1.0;      // No scale change
        pub const FOCUS_RING: f32 = 1.0;    // Focus ring offset
    }
}
```

## 7. Animation Choreography

### Stagger Patterns

For animating lists and grids:

```rust
pub mod tokens {
    pub mod stagger {
        pub const FAST: u32 = 30;          // 30ms between items
        pub const NORMAL: u32 = 50;        // 50ms between items
        pub const SLOW: u32 = 100;         // 100ms between items

        // Grid stagger (items animate in wave pattern)
        pub const GRID_STAGGER: StaggerConfig = StaggerConfig {
            delay: 30,
            from: StaggerFrom::Center,
        };
    }
}
```

### Enter/Exit Animations

Common animation patterns:

| Animation | Enter | Exit | Duration | Easing |
|-----------|-------|------|----------|--------|
| Fade | opacity 0→1 | opacity 1→0 | NORMAL | EASE_OUT |
| Slide Up | y: 20→0, opacity 0→1 | y: 0→-10, opacity 1→0 | SLOW | EASE_OUT |
| Slide Down | y: -20→0, opacity 0→1 | y: 0→10, opacity 1→0 | SLOW | EASE_OUT |
| Scale | scale 0.95→1, opacity 0→1 | scale 1→0.95, opacity 1→0 | NORMAL | SPRING_STIFF |
| Pop | scale 0.8→1 | scale 1→0.8, opacity 1→0 | NORMAL | SPRING_BOUNCY |
| Drawer | x: -100%→0 | x: 0→-100% | SLOW | EASE_OUT |
| Modal | scale 0.95→1, opacity 0→1 | scale 1→0.95, opacity 1→0 | SLOW | SPRING_STIFF |

## 8. Responsive Breakpoints

```rust
pub mod tokens {
    pub mod breakpoint {
        pub const SM: f32 = 640.0;      // Mobile landscape
        pub const MD: f32 = 768.0;      // Tablet portrait
        pub const LG: f32 = 1024.0;     // Tablet landscape / small desktop
        pub const XL: f32 = 1280.0;     // Desktop
        pub const XXL: f32 = 1536.0;    // Large desktop
    }
}
```

## 9. Z-Index Layers

```rust
pub mod tokens {
    pub mod z_index {
        pub const BELOW: i32 = -1;
        pub const BASE: i32 = 0;
        pub const RAISED: i32 = 1;
        pub const DROPDOWN: i32 = 10;
        pub const STICKY: i32 = 20;
        pub const FIXED: i32 = 30;
        pub const MODAL_BACKDROP: i32 = 40;
        pub const MODAL: i32 = 50;
        pub const POPOVER: i32 = 60;
        pub const TOOLTIP: i32 = 70;
        pub const TOAST: i32 = 80;
    }
}
```

## 10. Usage Examples

### Button with Motion

```blinc
@widget Button {
    @state pressed: bool = false

    @spring scale = 1.0 {
        config: tokens.spring.STIFF
        target: if pressed { tokens.scale.PRESSED } else { 1.0 }
    }

    @spring opacity = 1.0 {
        config: tokens.spring.SNAPPY
        target: if disabled { tokens.opacity.DISABLED } else { tokens.opacity.FULL }
    }

    @render {
        Container {
            transform: Transform::scale(scale)
            opacity: opacity
            background: tokens.color.PRIMARY_500
            border_radius: tokens.radius.MD
            padding: tokens.spacing.SPACE_4
            shadow: tokens.shadow.SM
            // ...
        }
    }
}
```

### Modal with Choreography

```blinc
@widget Modal {
    @state visible: bool = false

    @spring backdrop_opacity = 0.0 {
        config: tokens.spring.GENTLE
        target: if visible { 0.5 } else { 0.0 }
    }

    @spring content_scale = 0.95 {
        config: tokens.spring.STIFF
        target: if visible { 1.0 } else { 0.95 }
    }

    @spring content_opacity = 0.0 {
        config: tokens.spring.SNAPPY
        target: if visible { 1.0 } else { 0.0 }
    }

    @render {
        // Backdrop
        Container {
            opacity: backdrop_opacity
            background: Color::black()
            z_index: tokens.z_index.MODAL_BACKDROP
        }

        // Content
        Container {
            transform: Transform::scale(content_scale)
            opacity: content_opacity
            z_index: tokens.z_index.MODAL
            shadow: tokens.shadow.XXL
            // ...
        }
    }
}
```

## 11. Token File Organization

```
tokens/
├── color.rs         # Color palette and semantic colors
├── typography.rs    # Font sizes, weights, line heights
├── spacing.rs       # Spacing scale
├── radius.rs        # Border radius values
├── shadow.rs        # Shadow definitions
├── duration.rs      # Animation durations
├── easing.rs        # Easing curves
├── spring.rs        # Spring physics presets
├── opacity.rs       # Opacity values for states
├── z_index.rs       # Layer ordering
├── breakpoint.rs    # Responsive breakpoints
└── mod.rs           # Re-exports all tokens
```

## 12. DSL Extensibility

All tokens are designed to be extended and overridden from the Blinc DSL. This enables custom theming and UX behaviors without modifying core framework code.

### Defining Custom Tokens

```blinc
// my-theme.blinc - Custom token definitions

@tokens MyBrandTokens {
    // Extend color palette with brand colors
    @color brand_primary: #FF6B35
    @color brand_secondary: #004E89
    @color brand_accent: #F7C548

    // Custom semantic colors
    @color surface_elevated: #FFFFFF
    @color surface_sunken: #F5F5F5

    // Custom spacing scale
    @spacing card_padding: 24
    @spacing section_gap: 48

    // Custom border radius
    @radius card: 16
    @radius button: 8
    @radius pill: 9999
}
```

### Custom Spring Presets

Define springs with your own physics characteristics:

```blinc
@tokens MyMotionTokens {
    // Custom spring for card interactions
    @spring card_hover {
        stiffness: 350
        damping: 28
        mass: 1.0
    }

    // Playful bounce for notifications
    @spring notification_enter {
        stiffness: 450
        damping: 18
        mass: 0.8
    }

    // Smooth drawer slide
    @spring drawer {
        stiffness: 280
        damping: 32
        mass: 1.2
    }

    // Custom physics for drag interactions
    @spring drag_release {
        stiffness: 500
        damping: 35
        mass: 1.0
        velocity_threshold: 0.01  // Custom settling threshold
    }
}
```

### Custom Easing Curves

Define custom cubic bezier curves:

```blinc
@tokens MyEasingTokens {
    // Dramatic ease-out
    @easing dramatic_out: cubic_bezier(0.0, 0.0, 0.1, 1.0)

    // Snappy response
    @easing snappy: cubic_bezier(0.2, 0.8, 0.2, 1.0)

    // Elastic bounce
    @easing elastic: cubic_bezier(0.68, -0.55, 0.27, 1.55)

    // Anticipation before action
    @easing wind_up: cubic_bezier(0.36, 0.0, 0.66, -0.2)
}
```

### Custom Animation Presets

Create reusable animation sequences:

```blinc
@tokens MyAnimationTokens {
    // Page transition preset
    @animation page_enter {
        duration: 400ms
        easing: $dramatic_out
        properties: {
            opacity: 0 -> 1
            y: 30 -> 0
        }
    }

    // Success celebration
    @animation success_pop {
        duration: 300ms
        spring: $notification_enter
        properties: {
            scale: 0.8 -> 1.0
            opacity: 0 -> 1
        }
        then: {
            duration: 100ms
            properties: { scale: 1.0 -> 1.05 -> 1.0 }
        }
    }

    // Stagger pattern for lists
    @animation list_stagger {
        stagger: 40ms
        from: top
        properties: {
            opacity: 0 -> 1
            x: -20 -> 0
        }
    }
}
```

### Custom Interaction Behaviors

Define how widgets respond to user input:

```blinc
@tokens MyInteractionTokens {
    // Hover behavior
    @behavior hover_lift {
        spring: $card_hover
        properties: {
            y: 0 -> -4
            shadow: $shadow.md -> $shadow.xl
        }
    }

    // Press feedback
    @behavior press_sink {
        spring: tokens.spring.SNAPPY
        properties: {
            scale: 1.0 -> 0.97
            shadow: $shadow.md -> $shadow.sm
        }
    }

    // Focus ring
    @behavior focus_ring {
        duration: 150ms
        easing: $snappy
        properties: {
            ring_opacity: 0 -> 1
            ring_offset: 0 -> 2
        }
    }
}
```

### Theme Definition

Compose tokens into a complete theme:

```blinc
@theme MyAppTheme {
    // Import base tokens
    extends: tokens.default

    // Override colors
    colors: {
        primary: $brand_primary
        secondary: $brand_secondary
        accent: $brand_accent
        background: #FAFAFA
        surface: #FFFFFF
    }

    // Override motion
    motion: {
        spring_default: $card_hover
        duration_normal: 250ms
        reduce_motion: {
            // Accessibility: reduced motion preferences
            spring_default: tokens.spring.SNAPPY
            duration_normal: 0ms
        }
    }

    // Override typography
    typography: {
        font_family: "Inter, system-ui, sans-serif"
        font_family_mono: "JetBrains Mono, monospace"
        base_size: 16
    }

    // Override components
    components: {
        Button: {
            radius: $radius.button
            padding: [$spacing.card_padding / 2, $spacing.card_padding]
            hover_behavior: $hover_lift
            press_behavior: $press_sink
        }
        Card: {
            radius: $radius.card
            padding: $spacing.card_padding
            shadow: tokens.shadow.MD
        }
    }
}
```

### Using Custom Themes

Apply themes at the app or component level:

```blinc
// App-level theme
@app MyApp {
    theme: MyAppTheme

    @render {
        // All children inherit theme
        Router { ... }
    }
}

// Component-level theme override
@widget DarkModeSection {
    @render {
        ThemeProvider(theme: MyAppTheme.dark()) {
            // Children use dark variant
            Card { ... }
        }
    }
}

// Dynamic theme switching
@widget ThemeSwitcher {
    @state is_dark: bool = false

    @derived current_theme = if is_dark {
        MyAppTheme.dark()
    } else {
        MyAppTheme.light()
    }

    @render {
        ThemeProvider(theme: current_theme) {
            // Reactive theme changes with smooth transitions
            @children
        }
    }
}
```

### Token Inheritance & Composition

Tokens can inherit and compose from other tokens:

```blinc
@tokens ExtendedTokens {
    // Inherit all from base
    extends: tokens.default

    // Compose spring from existing values
    @spring responsive_spring {
        stiffness: tokens.spring.STIFF.stiffness * 1.2
        damping: tokens.spring.STIFF.damping
        mass: 0.9
    }

    // Color derived from primary
    @color primary_light: lighten($primary, 20%)
    @color primary_dark: darken($primary, 20%)

    // Computed spacing
    @spacing content_width: min(1200, 100% - $spacing.section_gap * 2)
}
```

### Runtime Token Access

Tokens can be accessed and modified at runtime:

```blinc
@widget DynamicTheme {
    @state accent_hue: f32 = 200.0

    // Computed token based on runtime state
    @derived dynamic_primary: Color = hsl($accent_hue, 70%, 50%)

    @render {
        ThemeProvider(
            overrides: {
                colors.primary: $dynamic_primary
                colors.primary_light: lighten($dynamic_primary, 20%)
            }
        ) {
            // Theme updates reactively when accent_hue changes
            ColorPicker(value: $accent_hue, on_change: |h| accent_hue = h)
            Button { label: "Themed Button" }
        }
    }
}
```

## 13. Accessibility & Motion Preferences

Tokens automatically respect system accessibility preferences:

```blinc
@tokens AccessibleMotion {
    // Standard motion
    @spring default {
        stiffness: 400
        damping: 30
        mass: 1.0
    }

    // When prefers-reduced-motion is set
    @media (prefers-reduced-motion: reduce) {
        @spring default {
            // Instant transitions, no animation
            stiffness: 10000
            damping: 1000
            mass: 1.0
        }

        @duration all: 0ms
    }
}
```

## 14. Theme System (Rust API)

For programmatic access, themes are also available as Rust structs:

```rust
pub struct Theme {
    pub colors: ColorTokens,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub shadows: ShadowTokens,
    pub motion: MotionTokens,
}

impl Theme {
    pub fn light() -> Self { /* ... */ }
    pub fn dark() -> Self { /* ... */ }
    pub fn custom(overrides: ThemeOverrides) -> Self { /* ... */ }

    /// Create theme from DSL token definitions
    pub fn from_blinc(tokens_file: &Path) -> Result<Self, ThemeError> { /* ... */ }
}
```

---

## Summary

This token system provides:

1. **Consistency** - All visual properties reference the same scale
2. **Flexibility** - Easy to create themes and variations
3. **Performance** - Springs provide 60fps+ physics-based animations
4. **Accessibility** - Proper contrast ratios, motion preferences
5. **Maintainability** - Single source of truth for design values
6. **DSL Extensibility** - Custom tokens, springs, easings, and behaviors defined in Blinc DSL
7. **Runtime Dynamism** - Reactive token updates for dynamic theming
8. **Composition** - Tokens can inherit, extend, and compute from other tokens
