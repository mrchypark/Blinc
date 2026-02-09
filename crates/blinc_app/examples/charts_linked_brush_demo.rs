//! blinc_charts Linked + Brush Demo
//!
//! Run with:
//! `cargo run -p blinc_app --example charts_linked_brush_demo --features windowed`
//!
//! Interactions:
//! - Drag: pan (shared X domain)
//! - Scroll / pinch: zoom (shared X domain)
//! - Shift + drag: brush-select X range (shared selection)
//!
//! Optional:
//! - Set `BLINC_CHARTS_N` to control the number of points (default: 200_000)

use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_charts::prelude::*;
use blinc_core::Color;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "blinc_charts: Linked + Brush Demo".to_string(),
        width: 1200,
        height: 820,
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
        .unwrap_or(200_000);

    let series_a = make_series(n, 1.0).expect("failed to create series (x must be sorted)");
    let series_b = make_series(n, 0.35).expect("failed to create series (x must be sorted)");

    let (x0, x1) = series_a.x_min_max();
    let link = chart_link(x0, x1);

    let handle_a = LineChartHandle::new(LineChartModel::new(series_a));
    let handle_b = LineChartHandle::new(LineChartModel::new(series_b));

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
                    text("blinc_charts: linked X-domain + brush selection")
                        .size(20.0)
                        .color(Color::rgba(0.95, 0.96, 0.98, 1.0)),
                )
                .child(
                    text(format!("N = {n} (env: BLINC_CHARTS_N)"))
                        .size(12.0)
                        .color(Color::rgba(0.7, 0.75, 0.82, 1.0)),
                )
                .child(
                    text("Drag: pan | Wheel/Pinch: zoom | Shift+Drag: brush")
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
                .child(linked_line_chart(handle_a, link.clone())),
        )
        .child(
            div()
                .flex_1()
                .rounded(14.0)
                .overflow_clip()
                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                .child(linked_line_chart(handle_b, link)),
        )
}

fn make_series(n: usize, amp: f32) -> anyhow::Result<TimeSeriesF32> {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);

    for i in 0..n {
        let t = i as f32 * 0.001;
        let v = (t * 1.0).sin() * 0.8 + (t * 0.13).sin() * 0.2 + (t.fract() - 0.5) * 0.05;
        x.push(i as f32);
        y.push(v * amp);
    }

    TimeSeriesF32::new(x, y)
}

