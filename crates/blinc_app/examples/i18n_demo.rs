//! i18n Demo (Simple + Fluent-compatible API)
//!
//! Run with:
//! `cargo run -p blinc_app --example i18n_demo --features windowed`

use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_i18n::{t, I18nState};
use blinc_layout::stateful::ButtonState;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Initialize catalogs before the first frame.
    if I18nState::try_get().is_none() {
        I18nState::init("en-US");
    }
    let i18n = I18nState::get();
    let simple_catalogs = [
        (
            "en-US",
            include_str!("../../../resource/i18n/demo.en-US.yaml"),
        ),
        (
            "ko-KR",
            include_str!("../../../resource/i18n/demo.ko-KR.yaml"),
        ),
    ];
    for (locale, content) in simple_catalogs {
        i18n.load_simple_catalog_str(locale, content)
            .map_err(|e| BlincError::Other(e.to_string()))?;
    }

    // Fluent catalogs are loaded via the same API surface. (Enabled by default in blinc_i18n.)
    let fluent_catalogs = [
        (
            "en-US",
            include_str!("../../../resource/i18n/demo.en-US.ftl"),
        ),
        (
            "ko-KR",
            include_str!("../../../resource/i18n/demo.ko-KR.ftl"),
        ),
    ];
    for (locale, content) in fluent_catalogs {
        i18n.load_fluent_ftl(locale, content)
            .map_err(|e| BlincError::Other(e.to_string()))?;
    }

    let config = WindowConfig {
        title: "Blinc i18n Demo".to_string(),
        width: 900,
        height: 600,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    let i18n = I18nState::get();
    let locale = i18n.locale();

    let toggle_btn = ctx.use_state_for("toggle_locale_btn", ButtonState::Idle);

    div()
        .w(ctx.width)
        .h(ctx.height)
        .p(32.0)
        .bg(Color::rgba(0.08, 0.09, 0.11, 1.0))
        .child(
            div()
                .glass()
                .rounded(18.0)
                .p(24.0)
                .gap(16.0)
                .flex_col()
                .child(text(t!("demo-title")).size(32.0).color(Color::WHITE))
                .child(
                    text(t!("demo-locale", { locale: locale.clone() }))
                        .size(16.0)
                        .color(Color::rgba(0.8, 0.85, 0.9, 1.0)),
                )
                .child(
                    text(t!("demo-hello", { name: "Chris" }))
                        .size(20.0)
                        .color(Color::rgba(0.9, 0.92, 0.95, 1.0)),
                )
                .child(
                    button(toggle_btn, t!("demo-toggle"))
                        .on_click(|_| {
                            let i18n = I18nState::get();
                            if i18n.locale() == "en-US" {
                                i18n.set_locale("ko-KR");
                            } else {
                                i18n.set_locale("en-US");
                            }
                        })
                        .bg_color(Color::rgba(0.25, 0.55, 0.95, 1.0))
                        .hover_color(Color::rgba(0.30, 0.60, 1.0, 1.0))
                        .pressed_color(Color::rgba(0.18, 0.45, 0.85, 1.0)),
                ),
        )
}
