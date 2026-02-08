//! Unified Styling API Demo
//!
//! Demonstrates all styling approaches in Blinc:
//! - `css!` macro: CSS-like syntax with hyphenated property names
//! - `style!` macro: Rust-friendly syntax with underscored names
//! - `ElementStyle` builder: Programmatic style construction
//! - CSS Parser: Runtime CSS string parsing
//!
//! All approaches produce `ElementStyle` - a unified schema for visual properties.
//!
//! Run with: cargo run -p blinc_app --example styling_demo --features windowed

use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_core::{Color, Shadow, Transform};
use blinc_layout::css;
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::element_style::ElementStyle;
use blinc_layout::style;
use blinc_theme::{ColorToken, ThemeState};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "Blinc Unified Styling API Demo".to_string(),
        width: 1000,
        height: 800,
        resizable: true,
        fullscreen: false,
        ..Default::default()
    };

    let mut css_loaded = false;

    WindowedApp::run(config, move |ctx| {
        // Load CSS stylesheet once — base styles, hover states, and animations
        // are applied automatically to elements with matching IDs.
        if !css_loaded {
            ctx.add_css(r#"
            #css-card {
                background: #3b82f6;
                border-radius: 12px;
                box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            }
            #css-card:hover {
                background: #60a5fa;
                box-shadow: 0 8px 16px rgba(59, 130, 246, 0.4);
            }

            #css-alert {
                background: #ef4444;
                border-radius: 8px;
                opacity: 0.95;
            }
            #css-alert:hover {
                opacity: 1.0;
                background: #f87171;
            }

            #css-glass {
                background: rgba(255, 255, 255, 0.15);
                border-radius: 16px;
                backdrop-filter: blur(10px);
            }

            #hover-blue {
                background: #3b82f6;
                border-radius: 8px;
            }
            #hover-blue:hover {
                background: #2563eb;
                box-shadow: 0 4px 12px rgba(37, 99, 235, 0.5);
            }

            #hover-green {
                background: #22c55e;
                border-radius: 8px;
            }
            #hover-green:hover {
                background: #16a34a;
                opacity: 0.9;
            }

            #hover-purple {
                background: #a855f7;
                border-radius: 12px;
            }
            #hover-purple:hover {
                background: #9333ea;
                box-shadow: 0 6px 20px rgba(147, 51, 234, 0.5);
            }

            #hover-orange {
                background: #f97316;
                border-radius: 16px;
            }
            #hover-orange:hover {
                background: #ea580c;
            }

            @keyframes pulse {
                0% { opacity: 0.5; }
                50% { opacity: 1.0; }
                100% { opacity: 0.5; }
            }
            #anim-pulse {
                background: #ec4899;
                border-radius: 8px;
                animation: pulse 2000ms ease-in-out infinite;
            }

            @keyframes glow {
                0% { opacity: 0.6; }
                50% { opacity: 1.0; }
                100% { opacity: 0.6; }
            }
            #anim-glow {
                background: #8b5cf6;
                border-radius: 12px;
                animation: glow 3000ms ease-in-out infinite;
            }
            "#);
            css_loaded = true;
        }

        build_ui(ctx)
    })
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    let theme = ThemeState::get();
    let bg = theme.color(ColorToken::Background);

    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(bg)
        .flex_col()
        .child(header())
        .child(
            scroll().w_full().h(ctx.height - 80.0).child(
                div()
                    .w_full()
                    .p(theme.spacing().space_6)
                    .flex_col()
                    .gap(theme.spacing().space_8)
                    // CSS Stylesheet integration (new!)
                    .child(css_stylesheet_section())
                    .child(css_hover_section())
                    .child(css_animation_section())
                    // Styling API sections
                    .child(css_macro_section())
                    .child(style_macro_section())
                    .child(builder_pattern_section())
                    .child(css_parser_section())
                    .child(style_merging_section())
                    .child(backgrounds_section())
                    .child(corner_radius_section())
                    .child(shadows_section())
                    .child(transforms_section())
                    .child(opacity_section())
                    .child(materials_section())
                    .child(api_comparison_section()),
            ),
        )
}

fn header() -> impl ElementBuilder {
    let theme = ThemeState::get();
    let surface = theme.color(ColorToken::Surface);
    let text_primary = theme.color(ColorToken::TextPrimary);
    let text_secondary = theme.color(ColorToken::TextSecondary);
    let border = theme.color(ColorToken::Border);

    div()
        .w_full()
        .h(80.0)
        .bg(surface)
        .border_bottom(1.0, border)
        .flex_row()
        .items_center()
        .justify_center()
        .gap(16.0)
        .child(
            text("Unified Styling API")
                .size(28.0)
                .weight(FontWeight::Bold)
                .color(text_primary),
        )
        .child(
            text("Stylesheets | Hover | Animations | css! | style! | CSS Parser")
                .size(14.0)
                .color(text_secondary),
        )
}

// ============================================================================
// Section Container Helpers
// ============================================================================

fn section_container() -> Div {
    let theme = ThemeState::get();
    let surface = theme.color(ColorToken::Surface);
    let border = theme.color(ColorToken::Border);

    div()
        .w_full()
        .bg(surface)
        .border(1.0, border)
        .rounded(12.0)
        .p(24.0)
        .flex_col()
        .gap(16.0)
}

fn section_title(title: &str) -> impl ElementBuilder {
    let theme = ThemeState::get();
    let text_primary = theme.color(ColorToken::TextPrimary);

    text(title)
        .size(20.0)
        .weight(FontWeight::SemiBold)
        .color(text_primary)
}

fn section_description(desc: &str) -> impl ElementBuilder {
    let theme = ThemeState::get();
    let text_secondary = theme.color(ColorToken::TextSecondary);

    text(desc).size(14.0).color(text_secondary)
}

fn code_label(label: &str) -> impl ElementBuilder {
    inline_code(label).size(12.0)
}

// ============================================================================
// CSS STYLESHEET SECTION (automatic style application via ctx.add_css)
// ============================================================================

fn css_stylesheet_section() -> impl ElementBuilder {
    let theme = ThemeState::get();
    let text_secondary = theme.color(ColorToken::TextSecondary);

    section_container()
        .child(section_title("CSS Stylesheet (ctx.add_css)"))
        .child(section_description(
            "Styles applied automatically via ctx.add_css(). Elements get #id selectors — no manual wiring needed.",
        ))
        .child(
            div()
                .flex_col()
                .gap(8.0)
                .child(
                    text("ctx.add_css(\"#css-card { background: #3b82f6; border-radius: 12px; ... }\")")
                        .size(12.0)
                        .color(text_secondary),
                )
                .child(
                    div()
                        .flex_row()
                        .flex_wrap()
                        .gap(16.0)
                        // Card styled by stylesheet
                        .child(
                            div()
                                .flex_col()
                                .gap(8.0)
                                .child(code_label("#css-card"))
                                .child(div().w(80.0).h(80.0).id("css-card")),
                        )
                        // Alert styled by stylesheet
                        .child(
                            div()
                                .flex_col()
                                .gap(8.0)
                                .child(code_label("#css-alert"))
                                .child(div().w(80.0).h(80.0).id("css-alert")),
                        )
                        // Glass styled by stylesheet
                        .child(
                            div()
                                .flex_col()
                                .gap(8.0)
                                .child(code_label("#css-glass"))
                                .child(
                                    div()
                                        .w(80.0)
                                        .h(80.0)
                                        .id("css-glass")
                                        .bg(Color::rgb(0.3, 0.4, 0.6)),
                                ),
                        ),
                ),
        )
}

// ============================================================================
// CSS HOVER SECTION (automatic :hover state styles)
// ============================================================================

fn css_hover_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("CSS :hover States"))
        .child(section_description(
            "Hover over boxes to see automatic :hover styles. Defined in stylesheet, applied by the framework.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#hover-blue"))
                        .child(div().w(80.0).h(80.0).id("hover-blue")),
                )
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#hover-green"))
                        .child(div().w(80.0).h(80.0).id("hover-green")),
                )
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#hover-purple"))
                        .child(div().w(80.0).h(80.0).id("hover-purple")),
                )
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#hover-orange"))
                        .child(div().w(80.0).h(80.0).id("hover-orange")),
                ),
        )
}

// ============================================================================
// CSS ANIMATION SECTION (@keyframes + animation property)
// ============================================================================

fn css_animation_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("CSS @keyframes Animations"))
        .child(section_description(
            "CSS animations via @keyframes. Defined in stylesheet, ticked automatically each frame.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#anim-pulse (2s infinite)"))
                        .child(div().w(80.0).h(80.0).id("anim-pulse")),
                )
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#anim-glow (3s infinite)"))
                        .child(div().w(80.0).h(80.0).id("anim-glow")),
                ),
        )
}

// ============================================================================
// CSS MACRO SECTION
// ============================================================================

fn css_macro_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("css! Macro"))
        .child(section_description(
            "CSS-like syntax with hyphenated property names and semicolon separators.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // Basic card with CSS properties
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("css! { background: ...; border-radius: ...; }"))
                        .child(styled_box_with_element_style(css! {
                            background: Color::BLUE;
                            border-radius: 8.0;
                            opacity: 0.9;
                        })),
                )
                // Shadow presets
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("css! { box-shadow: md; }"))
                        .child(styled_box_with_element_style(css! {
                            background: Color::WHITE;
                            border-radius: 12.0;
                            box-shadow: md;
                        })),
                )
                // Custom shadow
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("css! { box-shadow: Shadow::new(...); }"))
                        .child(styled_box_with_element_style(css! {
                            background: Color::GREEN;
                            border-radius: 8.0;
                            box-shadow: Shadow::new(4.0, 8.0, 12.0, Color::BLACK.with_alpha(0.3));
                        })),
                )
                // Backdrop filter (glass)
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("css! { backdrop-filter: glass; }"))
                        .child(styled_box_with_element_style(css! {
                            background: Color::WHITE.with_alpha(0.2);
                            border-radius: 16.0;
                            backdrop-filter: glass;
                        })),
                ),
        )
}

// ============================================================================
// STYLE MACRO SECTION
// ============================================================================

fn style_macro_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("style! Macro"))
        .child(section_description(
            "Rust-friendly syntax with underscored names and comma separators.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // Basic card
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("style! { bg: ..., rounded: ... }"))
                        .child(styled_box_with_element_style(style! {
                            bg: Color::PURPLE,
                            rounded: 8.0,
                            opacity: 0.9,
                        })),
                )
                // Preset methods
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("style! { rounded_lg, shadow_md }"))
                        .child(styled_box_with_element_style(style! {
                            bg: Color::WHITE,
                            rounded_lg,
                            shadow_md,
                        })),
                )
                // Transform shortcuts
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("style! { scale: 1.1 }"))
                        .child(styled_box_with_element_style(style! {
                            bg: Color::ORANGE,
                            rounded: 8.0,
                            scale: 1.1,
                        })),
                )
                // Material presets
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("style! { gold, rounded_xl }"))
                        .child(styled_box_with_element_style(style! {
                            bg: Color::from_hex(0xD4AF37), // Gold color
                            gold,
                            rounded_xl,
                        })),
                ),
        )
}

// ============================================================================
// BUILDER PATTERN SECTION
// ============================================================================

fn builder_pattern_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("ElementStyle Builder"))
        .child(section_description(
            "Programmatic construction using method chaining.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // Basic builder
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("ElementStyle::new().bg().rounded()"))
                        .child(styled_box_with_element_style(
                            ElementStyle::new().bg(Color::CYAN).rounded(8.0).shadow_sm(),
                        )),
                )
                // Advanced builder
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label(".rounded_corners().shadow_lg()"))
                        .child(styled_box_with_element_style(
                            ElementStyle::new()
                                .bg(Color::MAGENTA)
                                .rounded_corners(16.0, 16.0, 0.0, 0.0)
                                .shadow_lg(),
                        )),
                )
                // With transform
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label(".rotate_deg(10.0)"))
                        .child(styled_box_with_element_style(
                            ElementStyle::new()
                                .bg(Color::from_hex(0x008080)) // Teal
                                .rounded(12.0)
                                .rotate_deg(10.0),
                        )),
                )
                // With material
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label(".chrome().rounded(24.0)"))
                        .child(styled_box_with_element_style(
                            ElementStyle::new()
                                .bg(Color::from_hex(0xC0C0C8))
                                .chrome()
                                .rounded(24.0),
                        )),
                ),
        )
}

// ============================================================================
// CSS PARSER SECTION
// ============================================================================

fn css_parser_section() -> impl ElementBuilder {
    // Define CSS as a string using #id selectors
    let css_string = r#"
        #parser-card {
            background: #3b82f6;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }

        #parser-alert {
            background: #ef4444;
            border-radius: 8px;
            opacity: 0.95;
        }

        #parser-glass {
            background: rgba(255, 255, 255, 0.15);
            border-radius: 16px;
            backdrop-filter: blur(10px);
        }
    "#;

    // Parse at runtime
    let stylesheet = Stylesheet::parse(css_string).expect("valid CSS");
    let card_style = stylesheet.get("parser-card");
    let alert_style = stylesheet.get("parser-alert");
    let glass_style = stylesheet.get("parser-glass");

    section_container()
        .child(section_title("CSS Parser (Runtime)"))
        .child(section_description(
            "Parse CSS strings at runtime using Stylesheet::parse(). Uses #id selectors.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // Card style from CSS
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#parser-card { ... }"))
                        .child(if let Some(s) = card_style {
                            styled_box_with_element_style(s.clone())
                        } else {
                            styled_box_with_element_style(ElementStyle::new().bg(Color::GRAY))
                        }),
                )
                // Alert style from CSS
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#parser-alert { ... }"))
                        .child(if let Some(s) = alert_style {
                            styled_box_with_element_style(s.clone())
                        } else {
                            styled_box_with_element_style(ElementStyle::new().bg(Color::GRAY))
                        }),
                )
                // Glass style from CSS
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("#parser-glass { ... }"))
                        .child(if let Some(s) = glass_style {
                            styled_box_with_element_style(s.clone())
                        } else {
                            styled_box_with_element_style(ElementStyle::new().bg(Color::GRAY))
                        }),
                ),
        )
}

// ============================================================================
// STYLE MERGING SECTION
// ============================================================================

fn style_merging_section() -> impl ElementBuilder {
    // Base style
    let base = style! {
        bg: Color::BLUE,
        rounded: 12.0,
        shadow_md,
    };

    // Hover override
    let hover_overlay = style! {
        bg: Color::from_hex(0x3B82F6), // Lighter blue
        scale: 1.05,
    };

    // Merged result
    let merged = base.merge(&hover_overlay);

    section_container()
        .child(section_title("Style Merging"))
        .child(section_description(
            "Merge styles to create state-specific variants. Properties from overlay override base.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                .items_end()
                // Base style
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("Base style"))
                        .child(styled_box_with_element_style(base.clone())),
                )
                // Plus sign
                .child(
                    text("+")
                        .size(24.0)
                        .color(ThemeState::get().color(ColorToken::TextSecondary)),
                )
                // Hover overlay
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("Hover overlay"))
                        .child(styled_box_with_element_style(hover_overlay)),
                )
                // Equals sign
                .child(
                    text("=")
                        .size(24.0)
                        .color(ThemeState::get().color(ColorToken::TextSecondary)),
                )
                // Merged result
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("Merged result"))
                        .child(styled_box_with_element_style(merged)),
                ),
        )
}

// ============================================================================
// BACKGROUNDS SECTION
// ============================================================================

fn backgrounds_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("Backgrounds"))
        .child(section_description(
            "Solid colors with various construction methods.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // Solid color
                .child(labeled_box(
                    "Solid RED",
                    style! { bg: Color::RED, rounded: 8.0 },
                ))
                // With alpha
                .child(labeled_box(
                    "With Alpha",
                    style! { bg: Color::GREEN.with_alpha(0.6), rounded: 8.0 },
                ))
                // From hex
                .child(labeled_box(
                    "from_hex(0x9333EA)",
                    style! { bg: Color::from_hex(0x9333EA), rounded: 8.0 },
                ))
                // From hex (orange)
                .child(labeled_box(
                    "from_hex(0xF97316)",
                    style! { bg: Color::from_hex(0xF97316), rounded: 8.0 },
                ))
                // rgb() constructor
                .child(labeled_box(
                    "rgb(0.2, 0.6, 0.9)",
                    style! { bg: Color::rgb(0.2, 0.6, 0.9), rounded: 8.0 },
                )),
        )
}

// ============================================================================
// CORNER RADIUS SECTION
// ============================================================================

fn corner_radius_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("Corner Radius"))
        .child(section_description(
            "Uniform and per-corner radii with theme presets.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // None
                .child(labeled_box(
                    "rounded_none",
                    style! { bg: Color::BLUE, rounded_none },
                ))
                // Small
                .child(labeled_box(
                    "rounded_sm",
                    style! { bg: Color::BLUE, rounded_sm },
                ))
                // Medium
                .child(labeled_box(
                    "rounded_md",
                    style! { bg: Color::BLUE, rounded_md },
                ))
                // Large
                .child(labeled_box(
                    "rounded_lg",
                    style! { bg: Color::BLUE, rounded_lg },
                ))
                // XL
                .child(labeled_box(
                    "rounded_xl",
                    style! { bg: Color::BLUE, rounded_xl },
                ))
                // 2XL
                .child(labeled_box(
                    "rounded_2xl",
                    style! { bg: Color::BLUE, rounded_2xl },
                ))
                // Full (pill)
                .child(labeled_box(
                    "rounded_full",
                    style! { bg: Color::BLUE, rounded_full },
                ))
                // Custom per-corner
                .child(labeled_box(
                    "Top only",
                    style! { bg: Color::BLUE, rounded_corners: (16.0, 16.0, 0.0, 0.0) },
                ))
                // Custom uniform
                .child(labeled_box(
                    "rounded: 20.0",
                    css! { background: Color::BLUE; border-radius: 20.0; },
                )),
        )
}

// ============================================================================
// SHADOWS SECTION
// ============================================================================

fn shadows_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("Box Shadows"))
        .child(section_description(
            "Shadow presets (sm, md, lg, xl) and custom shadows.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(24.0)
                // Shadow presets
                .child(labeled_box(
                    "shadow_sm",
                    style! { bg: Color::WHITE, rounded: 8.0, shadow_sm },
                ))
                .child(labeled_box(
                    "shadow_md",
                    style! { bg: Color::WHITE, rounded: 8.0, shadow_md },
                ))
                .child(labeled_box(
                    "shadow_lg",
                    style! { bg: Color::WHITE, rounded: 8.0, shadow_lg },
                ))
                .child(labeled_box(
                    "shadow_xl",
                    style! { bg: Color::WHITE, rounded: 8.0, shadow_xl },
                ))
                // CSS syntax presets
                .child(labeled_box(
                    "box-shadow: md",
                    css! { background: Color::WHITE; border-radius: 8.0; box-shadow: md; },
                ))
                // Custom shadow
                .child(labeled_box(
                    "Custom shadow",
                    ElementStyle::new()
                        .bg(Color::WHITE)
                        .rounded(8.0)
                        .shadow(Shadow::new(8.0, 8.0, 16.0, Color::PURPLE.with_alpha(0.4))),
                ))
                // No shadow (explicit)
                .child(labeled_box(
                    "shadow_none",
                    style! { bg: Color::WHITE, rounded: 8.0, shadow_none },
                )),
        )
}

// ============================================================================
// TRANSFORMS SECTION
// ============================================================================

fn transforms_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("Transforms"))
        .child(section_description(
            "Scale, rotate, and translate transformations.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(32.0)
                // Scale up
                .child(labeled_box(
                    "scale: 1.2",
                    style! { bg: Color::GREEN, rounded: 8.0, scale: 1.2 },
                ))
                // Scale down
                .child(labeled_box(
                    "scale: 0.8",
                    style! { bg: Color::GREEN, rounded: 8.0, scale: 0.8 },
                ))
                // Non-uniform scale
                .child(labeled_box(
                    "scale_xy",
                    style! { bg: Color::GREEN, rounded: 8.0, scale_xy: (1.3, 0.8) },
                ))
                // Rotate
                .child(labeled_box(
                    "rotate_deg: 15",
                    style! { bg: Color::ORANGE, rounded: 8.0, rotate_deg: 15.0 },
                ))
                // Rotate negative
                .child(labeled_box(
                    "rotate_deg: -10",
                    style! { bg: Color::ORANGE, rounded: 8.0, rotate_deg: -10.0 },
                ))
                // Translate
                .child(labeled_box(
                    "translate: (10, 5)",
                    style! { bg: Color::PURPLE, rounded: 8.0, translate: (10.0, 5.0) },
                ))
                // CSS transform syntax
                .child(labeled_box(
                    "CSS transform",
                    css! {
                        background: Color::CYAN;
                        border-radius: 8.0;
                        transform: Transform::rotate(0.2);
                    },
                )),
        )
}

// ============================================================================
// OPACITY SECTION
// ============================================================================

fn opacity_section() -> impl ElementBuilder {
    let theme = ThemeState::get();
    let checkerboard = theme.color(ColorToken::SurfaceElevated);

    section_container()
        .child(section_title("Opacity"))
        .child(section_description(
            "Control element transparency with opacity values and presets.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // Show on checkerboard to demonstrate opacity
                .child(opacity_demo_box(
                    "opacity: 1.0",
                    style! { bg: Color::RED, rounded: 8.0, opacity: 1.0 },
                    checkerboard,
                ))
                .child(opacity_demo_box(
                    "opacity: 0.75",
                    style! { bg: Color::RED, rounded: 8.0, opacity: 0.75 },
                    checkerboard,
                ))
                .child(opacity_demo_box(
                    "opacity: 0.5",
                    style! { bg: Color::RED, rounded: 8.0, opacity: 0.5 },
                    checkerboard,
                ))
                .child(opacity_demo_box(
                    "opacity: 0.25",
                    style! { bg: Color::RED, rounded: 8.0, opacity: 0.25 },
                    checkerboard,
                ))
                .child(opacity_demo_box(
                    "opaque",
                    style! { bg: Color::BLUE, rounded: 8.0, opaque },
                    checkerboard,
                ))
                .child(opacity_demo_box(
                    "translucent",
                    style! { bg: Color::BLUE, rounded: 8.0, translucent },
                    checkerboard,
                ))
                .child(opacity_demo_box(
                    "transparent",
                    style! { bg: Color::BLUE, rounded: 8.0, transparent },
                    checkerboard,
                )),
        )
}

fn opacity_demo_box(label: &str, style: ElementStyle, bg: Color) -> impl ElementBuilder {
    div().flex_col().gap(8.0).child(code_label(label)).child(
        div()
            .w(80.0)
            .h(80.0)
            .bg(bg)
            .rounded(8.0)
            .items_center()
            .justify_center()
            .child(styled_box_with_element_style(style)),
    )
}

// ============================================================================
// MATERIALS SECTION
// ============================================================================

fn materials_section() -> impl ElementBuilder {
    section_container()
        .child(section_title("Materials"))
        .child(section_description(
            "Glass, metallic, chrome, gold, and wood effects (Blinc extensions).",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                // Glass
                .child(labeled_box(
                    "glass",
                    style! { bg: Color::WHITE.with_alpha(0.2), rounded: 16.0, glass },
                ))
                // Metallic
                .child(labeled_box(
                    "metallic",
                    style! { bg: Color::from_hex(0xB4B4BE), rounded: 8.0, metallic },
                ))
                // Chrome
                .child(labeled_box(
                    "chrome",
                    style! { bg: Color::from_hex(0xC8C8D2), rounded: 8.0, chrome },
                ))
                // Gold
                .child(labeled_box(
                    "gold",
                    style! { bg: Color::from_hex(0xD4AF37), rounded: 8.0, gold },
                ))
                // Wood
                .child(labeled_box(
                    "wood",
                    style! { bg: Color::from_hex(0x8B5A2B), rounded: 8.0, wood },
                ))
                // CSS backdrop-filter syntax
                .child(labeled_box(
                    "backdrop-filter: glass",
                    css! {
                        background: Color::WHITE.with_alpha(0.15);
                        border-radius: 16.0;
                        backdrop-filter: glass;
                    },
                )),
        )
}

// ============================================================================
// API COMPARISON SECTION
// ============================================================================

fn api_comparison_section() -> impl ElementBuilder {
    // Same visual result using all three approaches
    let from_css = css! {
        background: Color::from_hex(0x3B82F6);
        border-radius: 12.0;
        box-shadow: md;
        opacity: 0.95;
    };

    let from_style = style! {
        bg: Color::from_hex(0x3B82F6),
        rounded: 12.0,
        shadow_md,
        opacity: 0.95,
    };

    let from_builder = ElementStyle::new()
        .bg(Color::from_hex(0x3B82F6))
        .rounded(12.0)
        .shadow_md()
        .opacity(0.95);

    // CSS parser version
    let css_string = r#"
        .card {
            background: #3b82f6;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            opacity: 0.95;
        }
    "#;
    let stylesheet = Stylesheet::parse(css_string).expect("valid CSS");
    let from_parser = stylesheet.get("card").cloned().unwrap_or_default();

    section_container()
        .child(section_title("API Comparison"))
        .child(section_description(
            "All four approaches produce identical ElementStyle output.",
        ))
        .child(
            div()
                .flex_row()
                .flex_wrap()
                .gap(16.0)
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("css! { ... }"))
                        .child(styled_box_with_element_style(from_css)),
                )
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("style! { ... }"))
                        .child(styled_box_with_element_style(from_style)),
                )
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("ElementStyle::new()"))
                        .child(styled_box_with_element_style(from_builder)),
                )
                .child(
                    div()
                        .flex_col()
                        .gap(8.0)
                        .child(code_label("Stylesheet::parse()"))
                        .child(styled_box_with_element_style(from_parser)),
                ),
        )
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a styled box that applies ElementStyle properties
fn styled_box_with_element_style(es: ElementStyle) -> Div {
    div().w(80.0).h(80.0).style(&es)
}

/// Create a labeled demo box
fn labeled_box(label: &str, style: ElementStyle) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(8.0)
        .child(code_label(label))
        .child(styled_box_with_element_style(style))
}
