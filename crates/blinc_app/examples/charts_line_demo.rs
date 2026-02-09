//! blinc_charts Line Demo
//!
//! Run with:
//! `cargo run -p blinc_app --example charts_line_demo --features windowed`
//!
//! Optional:
//! - Set `BLINC_CHARTS_N` to control the number of points (default: 1_000_000)

use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_charts::prelude::*;
use blinc_core::Color;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "blinc_charts: Time-Series Line Demo".to_string(),
        width: 1200,
        height: 720,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    let n = std::env::var("BLINC_CHARTS_N")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v >= 2)
        .unwrap_or(1_000_000);

    let series = make_series(n).expect("failed to create series (x must be sorted)");
    let handle = LineChartHandle::new(LineChartModel::new(series));

    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .flex_col()
        .gap(10.0)
        .p(12.0)
        .child(
            div()
                .flex_row()
                .items_center()
                .gap(12.0)
                .child(
                    text("blinc_charts: ultra-large time-series line")
                        .size(20.0)
                        .color(Color::rgba(0.95, 0.96, 0.98, 1.0)),
                )
                .child(
                    text(format!("N = {n} (set BLINC_CHARTS_N to change)"))
                        .size(12.0)
                        .color(Color::rgba(0.7, 0.75, 0.82, 1.0)),
                ),
        )
        .child(
            div()
                .flex_1()
                .rounded(14.0)
                .overflow_clip()
                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                .child(line_chart(handle)),
        )
}

fn make_series(n: usize) -> anyhow::Result<TimeSeriesF32> {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);

    // Deterministic “no-alloc RNG-ish” signal: mixed sine waves + a tiny sawtooth.
    for i in 0..n {
        let t = i as f32 * 0.001;
        let v = (t * 1.0).sin() * 0.8 + (t * 0.13).sin() * 0.2 + (t.fract() - 0.5) * 0.05;
        x.push(i as f32);
        y.push(v);
    }

    TimeSeriesF32::new(x, y)
}
