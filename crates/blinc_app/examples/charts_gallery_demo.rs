//! blinc_charts Gallery Demo
//!
//! Run with:
//! `cargo run -p blinc_app --example charts_gallery_demo --features windowed`
//!
//! This demo is a living gallery: every chart candidate gets at least one section.
//! Some sections are placeholders until their dedicated chart types land.

use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_charts::prelude::*;
use blinc_core::{Brush, Color, DrawContext, Point, Rect, TextStyle};
use blinc_layout::prelude::Scroll;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "blinc_charts: Gallery".to_string(),
        width: 1200,
        height: 840,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    // Core datasets (kept deterministic; no RNG needed).
    let line_n = std::env::var("BLINC_CHARTS_N")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v >= 2)
        .unwrap_or(200_000);

    let line_series = make_series(line_n).expect("failed to create series (x must be sorted)");
    let line_handle = LineChartHandle::new(LineChartModel::new(line_series.clone()));

    let area_handle = AreaChartHandle::new(AreaChartModel::new(line_series.clone()));

    let scatter_handle = ScatterChartHandle::new(ScatterChartModel::new(line_series.clone()));

    let multi_handle = MultiLineChartHandle::new(
        MultiLineChartModel::new(make_multi_series(64, 240)).expect("multiline requires series"),
    );

    let bar_handle = BarChartHandle::new(
        BarChartModel::new(make_bar_series(3, 3_000)).expect("bar requires series"),
    );

    let hist_handle =
        HistogramChartHandle::new(HistogramChartModel::new(make_hist_values(100_000)).unwrap());

    let candle_handle = CandlestickChartHandle::new(CandlestickChartModel::new(
        CandleSeries::new(make_candles(120_000)).expect("candles must be sorted"),
    ));

    let heat_handle = HeatmapChartHandle::new(
        HeatmapChartModel::new(320, 160, make_heat_values(320, 160)).expect("valid heatmap"),
    );

    let header = div()
        .flex_row()
        .items_end()
        .justify_between()
        .child(
            text("blinc_charts gallery")
                .size(24.0)
                .weight(FontWeight::Bold)
                .color(Color::rgba(0.95, 0.96, 0.98, 1.0)),
        )
        .child(
            text(format!("BLINC_CHARTS_N = {line_n}"))
                .size(12.0)
                .color(Color::rgba(0.70, 0.75, 0.82, 1.0)),
        );

    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .p(12.0)
        .flex_col()
        .gap(12.0)
        .child(header)
        .child(
            Scroll::new()
                .w_full()
                .h_full()
                .rounded(14.0)
                .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
                .p(10.0)
                .child(
                    div()
                        .flex_col()
                        .gap(14.0)
                        .child(section(
                            "Line",
                            "Ultra-large time series line (LOD downsampling + GPU polyline path).",
                            div()
                                .h(320.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(line_chart(line_handle)),
                        ))
                        .child(section(
                            "Multi-line",
                            "Many series with gap breaks + global segment budgeting.",
                            div()
                                .h(320.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(multi_line_chart(multi_handle)),
                        ))
                        .child(section(
                            "Area",
                            "Single-series area fill + outline (same LOD as line).",
                            div()
                                .h(300.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(area_chart(area_handle)),
                        ))
                        .child(section(
                            "Bar / Stacked bar",
                            "Stacked bars with screen-binned aggregation (mean per x-bin).",
                            div()
                                .h(300.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(bar_chart(bar_handle)),
                        ))
                        .child(section(
                            "Histogram",
                            "Pre-binned histogram (demo: global bins).",
                            div()
                                .h(260.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(histogram_chart(hist_handle)),
                        ))
                        .child(section(
                            "Scatter / Bubble",
                            "Scatter using min/max downsample per x-bin (capped point count).",
                            div()
                                .h(300.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(scatter_chart(scatter_handle)),
                        ))
                        .child(section(
                            "Candlestick (OHLC)",
                            "Candles resampled to screen bins, then drawn as wick+body.",
                            div()
                                .h(320.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(candlestick_chart(candle_handle)),
                        ))
                        .child(section(
                            "Heatmap (2D bins)",
                            "Grid heatmap screen-sampled to keep cost bounded.",
                            div()
                                .h(320.0)
                                .rounded(14.0)
                                .overflow_clip()
                                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                                .child(heatmap_chart(heat_handle)),
                        ))
                        // Placeholders for remaining candidates (canvas-first).
                        .child(section(
                            "Area (stacked)",
                            "Placeholder: stacked area renderer (will share binning/LOD with multi-line).",
                            todo_canvas("TODO: stacked area"),
                        ))
                        .child(section(
                            "Density map / patch_map",
                            "Placeholder: integrate existing patch_map implementation as a linked chart view.",
                            todo_canvas("TODO: patch_map chart adapter"),
                        ))
                        .child(section(
                            "Contour / Isobands",
                            "Placeholder: contouring over 2D bins.",
                            todo_canvas("TODO: contours"),
                        ))
                        .child(section(
                            "Boxplot / Violin / Error bands",
                            "Placeholder: analytics overlays and distribution charts.",
                            todo_canvas("TODO: box/violin/error bands"),
                        ))
                        .child(section(
                            "Treemap / Sunburst / Icicle / Packing",
                            "Placeholder: hierarchy layout + canvas renderer.",
                            todo_canvas("TODO: hierarchy"),
                        ))
                        .child(section(
                            "Graph / Sankey / Chord",
                            "Placeholder: node-link and flow layouts.",
                            todo_canvas("TODO: network/flow"),
                        ))
                        .child(section(
                            "Parallel coordinates / Polar / Radar",
                            "Placeholder: alternative coordinate systems.",
                            todo_canvas("TODO: coords"),
                        ))
                        .child(section(
                            "Gauge / Funnel / Streamgraph",
                            "Placeholder: specialized series types.",
                            todo_canvas("TODO: specialized"),
                        ))
                        .child(section(
                            "Geo",
                            "Placeholder: projections + tile/image layers.",
                            todo_canvas("TODO: geo"),
                        )),
                ),
        )
}

fn section(title: &str, desc: &str, content: impl ElementBuilder + 'static) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(8.0)
        .child(
            div()
                .flex_col()
                .gap(2.0)
                .child(
                    text(title)
                        .size(16.0)
                        .weight(FontWeight::SemiBold)
                        .color(Color::rgba(0.92, 0.93, 0.95, 1.0)),
                )
                .child(
                    text(desc)
                        .size(12.0)
                        .color(Color::rgba(0.68, 0.72, 0.78, 1.0)),
                ),
        )
        .child(content)
}

fn todo_canvas(label: &'static str) -> impl ElementBuilder {
    div()
        .h(220.0)
        .rounded(14.0)
        .overflow_clip()
        .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
        .child(
            blinc_layout::canvas::canvas(move |ctx: &mut dyn DrawContext, bounds| {
                ctx.fill_rect(
                    Rect::new(0.0, 0.0, bounds.width, bounds.height),
                    0.0.into(),
                    Brush::Solid(Color::rgba(0.08, 0.09, 0.11, 1.0)),
                );
                let style = TextStyle::new(14.0).with_color(Color::rgba(1.0, 1.0, 1.0, 0.75));
                ctx.draw_text(label, Point::new(14.0, 14.0), &style);
            })
            .w_full()
            .h_full(),
        )
}

fn make_series(n: usize) -> anyhow::Result<TimeSeriesF32> {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 * 0.001;
        let v = (t * 1.0).sin() * 0.8 + (t * 0.13).sin() * 0.2 + (t.fract() - 0.5) * 0.05;
        x.push(i as f32);
        y.push(v);
    }
    TimeSeriesF32::new(x, y)
}

fn make_multi_series(series_n: usize, points_n: usize) -> Vec<TimeSeriesF32> {
    let mut out = Vec::with_capacity(series_n);
    for s in 0..series_n {
        let mut x = Vec::with_capacity(points_n);
        let mut y = Vec::with_capacity(points_n);
        let phase = s as f32 * 0.37;
        let mut cur_x = 0.0f32;
        for i in 0..points_n {
            // Force occasional gaps while keeping x sorted (monotonic).
            cur_x += if i % 37 == 0 && i != 0 { 9.0 } else { 1.0 };
            x.push(cur_x);
            let t = i as f32 * 0.06;
            let vv = (t + phase).sin() * 0.7 + (t * 0.17 + phase).sin() * 0.2;
            y.push(vv);
        }
        out.push(TimeSeriesF32::new(x, y).expect("sorted by construction"));
    }
    out
}

fn make_bar_series(series_n: usize, n: usize) -> Vec<TimeSeriesF32> {
    let mut out = Vec::with_capacity(series_n);
    for s in 0..series_n {
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        let phase = s as f32 * 0.85;
        for i in 0..n {
            let t = i as f32 * 0.01;
            let v = ((t + phase).sin() * 0.5 + 0.6).max(0.0);
            x.push(i as f32);
            y.push(v);
        }
        out.push(TimeSeriesF32::new(x, y).expect("sorted by construction"));
    }
    out
}

fn make_hist_values(n: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(n);
    // Two-lobe distribution (deterministic).
    for i in 0..n {
        let t = i as f32 * 0.001;
        let a = (t * 1.7).sin() * 0.6 + (t * 0.11).sin() * 0.15;
        let b = (t * 0.9).cos() * 0.4;
        out.push(a + b);
    }
    out
}

fn make_candles(n: usize) -> Vec<Candle> {
    let mut out = Vec::with_capacity(n);
    let mut last = 0.0f32;
    for i in 0..n {
        let t = i as f32 * 0.01;
        let drift = (t * 0.23).sin() * 0.02;
        let noise = (t * 1.7).sin() * 0.03 + (t * 2.1).cos() * 0.01;
        let close = last + drift + noise;
        let open = last;
        let hi = open.max(close) + (t * 0.7).sin().abs() * 0.04;
        let lo = open.min(close) - (t * 0.9).cos().abs() * 0.04;
        out.push(Candle {
            x: i as f32,
            open,
            high: hi,
            low: lo,
            close,
        });
        last = close;
    }
    out
}

fn make_heat_values(w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let cx = (w as f32) * 0.55;
    let cy = (h as f32) * 0.45;
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - cx) / (w as f32);
            let dy = (y as f32 - cy) / (h as f32);
            let r2 = dx * dx + dy * dy;
            let v = (-r2 * 38.0).exp() + (dx * 18.0).sin() * (dy * 14.0).cos() * 0.15;
            out[y * w + x] = v;
        }
    }
    out
}
