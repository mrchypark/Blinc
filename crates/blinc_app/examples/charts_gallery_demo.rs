//! blinc_charts Gallery Demo
//!
//! Run with:
//! `cargo run -p blinc_app --example charts_gallery_demo --features windowed`
//!
//! This demo is a living gallery for every chart in `blinc_charts`.
//! It also includes control buttons for runtime reconfiguration and seeded data randomization.

use blinc_app::demos::charts_gallery::{layout_mode, parse_initial_selected, LayoutMode};
use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_charts::prelude::*;
use blinc_core::{Color, Point, State};
use blinc_layout::prelude::{ButtonState, NoState};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChartKind {
    Line,
    MultiLine,
    Area,
    Bar,
    Histogram,
    Scatter,
    Candlestick,
    Heatmap,
    StackedArea,
    DensityMap,
    Contour,
    Statistics,
    Hierarchy,
    Network,
    Polar,
    Gauge,
    Funnel,
    Geo,
}

impl ChartKind {
    fn key(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::MultiLine => "multi_line",
            Self::Area => "area",
            Self::Bar => "bar",
            Self::Histogram => "histogram",
            Self::Scatter => "scatter",
            Self::Candlestick => "candlestick",
            Self::Heatmap => "heatmap",
            Self::StackedArea => "stacked_area",
            Self::DensityMap => "density_map",
            Self::Contour => "contour",
            Self::Statistics => "statistics",
            Self::Hierarchy => "hierarchy",
            Self::Network => "network",
            Self::Polar => "polar",
            Self::Gauge => "gauge",
            Self::Funnel => "funnel",
            Self::Geo => "geo",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ChartEntry {
    kind: ChartKind,
    group: &'static str,
    title: &'static str,
    subtitle: &'static str,
    usage: &'static str,
    controls: &'static str,
}

const fn chart_entry(
    kind: ChartKind,
    group: &'static str,
    title: &'static str,
    subtitle: &'static str,
    usage: &'static str,
    controls: &'static str,
) -> ChartEntry {
    ChartEntry {
        kind,
        group,
        title,
        subtitle,
        usage,
        controls,
    }
}

const ITEMS: &[ChartEntry] = &[
    chart_entry(
        ChartKind::Line,
        "Time Series",
        "Line",
        "Ultra-large single time series",
        "Best for trend + local anomaly checks over long ranges.",
        "Try preset changes to compare downsampling quality/perf.",
    ),
    chart_entry(
        ChartKind::MultiLine,
        "Time Series",
        "Multi-line",
        "Many series with intentional gaps",
        "Best for correlated-series comparisons and missing-data gaps.",
        "Use randomize to regenerate phase offsets and gap patterns.",
    ),
    chart_entry(
        ChartKind::Area,
        "Time Series",
        "Area",
        "Single series area fill",
        "Best for showing cumulative intensity and baseline distance.",
        "Switch preset to tune sampling density.",
    ),
    chart_entry(
        ChartKind::Bar,
        "Time Series",
        "Bar",
        "Screen-binned bars",
        "Best for categorical/value magnitude over sequential buckets.",
        "Use mode button to switch stacked/grouped rendering.",
    ),
    chart_entry(
        ChartKind::Histogram,
        "Distribution",
        "Histogram",
        "Pre-binned value distribution",
        "Best for quickly checking spread/skew/multi-modality.",
        "Randomize regenerates the source distribution.",
    ),
    chart_entry(
        ChartKind::Scatter,
        "Distribution",
        "Scatter",
        "Point cloud with primitive cap",
        "Best for dense trend + outlier exploration in sampled points.",
        "Preset controls point budget to match hardware limits.",
    ),
    chart_entry(
        ChartKind::Candlestick,
        "Financial",
        "Candlestick",
        "OHLC candles with screen binning",
        "Best for price-action inspection with open/high/low/close semantics.",
        "Randomize regenerates volatility and drift profile.",
    ),
    chart_entry(
        ChartKind::Heatmap,
        "Field / Grid",
        "Heatmap",
        "2D scalar grid",
        "Best for matrix-like intensity fields on fixed dimensions.",
        "Preset controls max sampled cells in each axis.",
    ),
    chart_entry(
        ChartKind::StackedArea,
        "Field / Grid",
        "Stacked Area",
        "Aligned multi-series area",
        "Best for part-to-whole temporal composition analysis.",
        "Use mode button to switch Stacked vs Streamgraph.",
    ),
    chart_entry(
        ChartKind::DensityMap,
        "Field / Grid",
        "Density Map",
        "2D histogram density",
        "Best for very dense scatter populations where raw points saturate.",
        "Preset adjusts cell and point budgets for responsiveness.",
    ),
    chart_entry(
        ChartKind::Contour,
        "Field / Grid",
        "Contour",
        "Marching-squares isobands",
        "Best for topology-level shape trends over scalar fields.",
        "Preset changes contour level depth and segment budget.",
    ),
    chart_entry(
        ChartKind::Statistics,
        "Statistical",
        "Statistics",
        "Boxplot-like grouped summaries",
        "Best for robust per-group distribution summaries.",
        "Randomize regenerates group spread/shift patterns.",
    ),
    chart_entry(
        ChartKind::Hierarchy,
        "Structural",
        "Hierarchy",
        "Treemap / Icicle / Sunburst / Packing",
        "Best for nested contribution analysis.",
        "Cycle layout mode to compare structural readability.",
    ),
    chart_entry(
        ChartKind::Network,
        "Structural",
        "Network",
        "Graph / Sankey / Chord",
        "Best for relationship flow and connectivity views.",
        "Use mode button to switch topology projection.",
    ),
    chart_entry(
        ChartKind::Polar,
        "Specialized",
        "Polar",
        "Radar / Polar / Parallel",
        "Best for dimension-wise profile comparisons.",
        "Cycle mode for different radial/parallel framing.",
    ),
    chart_entry(
        ChartKind::Gauge,
        "Specialized",
        "Gauge",
        "Single KPI dial",
        "Best for target-tracking single metric snapshots.",
        "Randomize current updates value while keeping scale.",
    ),
    chart_entry(
        ChartKind::Funnel,
        "Specialized",
        "Funnel",
        "Stage conversion funnel",
        "Best for conversion drop-off visualization.",
        "Randomize current regenerates stage retention profile.",
    ),
    chart_entry(
        ChartKind::Geo,
        "Specialized",
        "Geo",
        "Shape/polyline map-like rendering",
        "Best for path/region overlays with pan+zoom.",
        "Randomize current perturbs coastline/island geometry.",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataPreset {
    Fast,
    Balanced,
    Detail,
}

impl DataPreset {
    fn label(self) -> &'static str {
        match self {
            Self::Fast => "Fast",
            Self::Balanced => "Balanced",
            Self::Detail => "Detail",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Fast => Self::Balanced,
            Self::Balanced => Self::Detail,
            Self::Detail => Self::Fast,
        }
    }
}

#[derive(Clone, Debug)]
struct ThemePalette {
    bg: Color,
    grid: Color,
    text: Color,
    a: Color,
    b: Color,
    c: Color,
}

#[derive(Clone, Debug)]
struct GalleryConfig {
    line_n: usize,
    initial_seed: u64,
    base_seed: u64,
    chart_salts: Vec<u64>,

    preset: DataPreset,
    noise_enabled: bool,
    option_indices: BTreeMap<String, usize>,

    bar_stacked: bool,
    stacked_mode: StackedAreaMode,
    hierarchy_mode: HierarchyMode,
    network_mode: NetworkMode,
    polar_mode: PolarChartMode,
}

impl GalleryConfig {
    fn new(line_n: usize, item_count: usize) -> Self {
        let seed = parse_env_seed().unwrap_or(0xC0DEC0DE_u64);
        Self {
            line_n,
            initial_seed: seed,
            base_seed: seed,
            chart_salts: vec![0; item_count],
            preset: DataPreset::Balanced,
            noise_enabled: true,
            option_indices: BTreeMap::new(),
            bar_stacked: true,
            stacked_mode: StackedAreaMode::Stacked,
            hierarchy_mode: HierarchyMode::Treemap,
            network_mode: NetworkMode::Graph,
            polar_mode: PolarChartMode::Radar,
        }
    }

    fn seed_for_index(&self, idx: usize) -> u64 {
        let salt = self.chart_salts.get(idx).copied().unwrap_or_default();
        splitmix64(self.base_seed ^ salt ^ ((idx as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)))
    }

    fn randomize_all(&mut self) {
        self.base_seed = splitmix64(self.base_seed.wrapping_add(0xA076_1D64_78BD_642F));
    }

    fn randomize_chart(&mut self, idx: usize) {
        if let Some(salt) = self.chart_salts.get_mut(idx) {
            *salt =
                splitmix64(salt.wrapping_add((idx as u64 + 1).wrapping_mul(0xBF58_476D_1CE4_E5B9)));
        }
    }

    fn option_index(&self, kind: ChartKind, name: &str, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        let key = format!("{}:{name}", kind.key());
        self.option_indices.get(&key).copied().unwrap_or(0) % len
    }

    fn pick_usize(&self, kind: ChartKind, name: &str, values: &[usize]) -> usize {
        values[self.option_index(kind, name, values.len())]
    }

    fn pick_f32(&self, kind: ChartKind, name: &str, values: &[f32]) -> f32 {
        values[self.option_index(kind, name, values.len())]
    }

    fn theme_palette(&self, kind: ChartKind) -> Option<ThemePalette> {
        match self.option_index(kind, "theme", 4) {
            0 => None,
            1 => Some(ThemePalette {
                bg: Color::rgba(0.05, 0.07, 0.11, 1.0),
                grid: Color::rgba(0.35, 0.55, 0.72, 0.22),
                text: Color::rgba(0.86, 0.93, 0.98, 1.0),
                a: Color::rgba(0.40, 0.78, 1.0, 1.0),
                b: Color::rgba(0.17, 0.52, 0.82, 0.62),
                c: Color::rgba(0.82, 0.94, 1.0, 0.75),
            }),
            2 => Some(ThemePalette {
                bg: Color::rgba(0.10, 0.08, 0.06, 1.0),
                grid: Color::rgba(0.62, 0.46, 0.24, 0.24),
                text: Color::rgba(0.98, 0.92, 0.82, 1.0),
                a: Color::rgba(1.0, 0.67, 0.30, 1.0),
                b: Color::rgba(0.85, 0.40, 0.20, 0.58),
                c: Color::rgba(1.0, 0.86, 0.60, 0.78),
            }),
            _ => Some(ThemePalette {
                bg: Color::rgba(0.07, 0.07, 0.08, 1.0),
                grid: Color::rgba(0.62, 0.64, 0.68, 0.18),
                text: Color::rgba(0.93, 0.94, 0.96, 1.0),
                a: Color::rgba(0.88, 0.92, 0.98, 1.0),
                b: Color::rgba(0.66, 0.71, 0.80, 0.55),
                c: Color::rgba(0.95, 0.97, 1.0, 0.70),
            }),
        }
    }

    fn cycle_option(&mut self, kind: ChartKind, name: &str, len: usize) {
        if len == 0 {
            return;
        }
        let key = format!("{}:{name}", kind.key());
        let cur = self.option_indices.get(&key).copied().unwrap_or(0);
        self.option_indices.insert(key, (cur + 1) % len);
    }

    fn reset(&mut self) {
        self.base_seed = self.initial_seed;
        self.chart_salts.fill(0);

        self.preset = DataPreset::Balanced;
        self.noise_enabled = true;
        self.option_indices.clear();
        self.bar_stacked = true;
        self.stacked_mode = StackedAreaMode::Stacked;
        self.hierarchy_mode = HierarchyMode::Treemap;
        self.network_mode = NetworkMode::Graph;
        self.polar_mode = PolarChartMode::Radar;
    }
}

#[derive(Clone)]
struct GalleryModels {
    line: LineChartHandle,
    multi: MultiLineChartHandle,
    area: AreaChartHandle,
    bar: BarChartHandle,
    hist: HistogramChartHandle,
    scatter: ScatterChartHandle,
    candle: CandlestickChartHandle,
    heat: HeatmapChartHandle,
    stacked_area: StackedAreaChartHandle,
    density: DensityMapChartHandle,
    contour: ContourChartHandle,
    stats: StatisticsChartHandle,
    hierarchy: HierarchyChartHandle,
    network: NetworkChartHandle,
    polar: PolarChartHandle,
    gauge: GaugeChartHandle,
    funnel: FunnelChartHandle,
    geo: GeoChartHandle,
}

impl GalleryModels {
    fn new(config: &GalleryConfig) -> Self {
        set_gallery_noise_enabled(config.noise_enabled);

        let line_seed = config.seed_for_index(ChartKind::Line as usize);
        let multi_seed = config.seed_for_index(ChartKind::MultiLine as usize);
        let bar_seed = config.seed_for_index(ChartKind::Bar as usize);
        let hist_seed = config.seed_for_index(ChartKind::Histogram as usize);
        let scatter_seed = config.seed_for_index(ChartKind::Scatter as usize);
        let candle_seed = config.seed_for_index(ChartKind::Candlestick as usize);
        let heat_seed = config.seed_for_index(ChartKind::Heatmap as usize);
        let stacked_seed = config.seed_for_index(ChartKind::StackedArea as usize);
        let density_seed = config.seed_for_index(ChartKind::DensityMap as usize);
        let contour_seed = config.seed_for_index(ChartKind::Contour as usize);
        let stats_seed = config.seed_for_index(ChartKind::Statistics as usize);
        let hierarchy_seed = config.seed_for_index(ChartKind::Hierarchy as usize);
        let network_seed = config.seed_for_index(ChartKind::Network as usize);
        let polar_seed = config.seed_for_index(ChartKind::Polar as usize);
        let gauge_seed = config.seed_for_index(ChartKind::Gauge as usize);
        let funnel_seed = config.seed_for_index(ChartKind::Funnel as usize);
        let geo_seed = config.seed_for_index(ChartKind::Geo as usize);

        let line_series = make_series(config.line_n, line_seed)
            .expect("failed to create line series (x must be sorted)");

        let line = LineChartHandle::new(LineChartModel::new(line_series.clone()));
        let area = AreaChartHandle::new(AreaChartModel::new(line_series.clone()));
        let scatter = ScatterChartHandle::new(ScatterChartModel::new(
            make_series(config.line_n, scatter_seed).expect("scatter series"),
        ));

        let mut multi_model = MultiLineChartModel::new(make_multi_series(72, 240, multi_seed))
            .expect("multiline requires series");
        multi_model.set_gap_dx(seed_range(multi_seed, 99, 4.0, 14.0));
        let multi = MultiLineChartHandle::new(multi_model);

        let mut bar_model =
            BarChartModel::new(make_bar_series(4, 3_200, bar_seed)).expect("bar requires series");
        bar_model.style.stacked = config.bar_stacked;
        let bar = BarChartHandle::new(bar_model);

        let hist = HistogramChartHandle::new(
            HistogramChartModel::new(make_hist_values(100_000, hist_seed)).expect("hist values"),
        );

        let candle = CandlestickChartHandle::new(CandlestickChartModel::new(
            CandleSeries::new(make_candles(120_000, candle_seed)).expect("candles must be sorted"),
        ));

        let heat = HeatmapChartHandle::new(
            HeatmapChartModel::new(320, 160, make_heat_values(320, 160, heat_seed))
                .expect("valid heatmap"),
        );

        let mut stacked_model = StackedAreaChartModel::new(make_stacked_series(
            6,
            (config.line_n / 2).max(20_000),
            stacked_seed,
        ))
        .expect("stacked area requires aligned series");
        stacked_model.style.mode = config.stacked_mode;
        let stacked_area = StackedAreaChartHandle::new(stacked_model);

        let density = DensityMapChartHandle::new(
            DensityMapChartModel::new(make_density_points(
                (config.line_n / 2).max(60_000),
                density_seed,
            ))
            .expect("density requires points"),
        );

        let contour = ContourChartHandle::new(
            ContourChartModel::new(240, 120, make_contour_values(240, 120, contour_seed))
                .expect("contour grid"),
        );

        let stats = StatisticsChartHandle::new(
            StatisticsChartModel::new(make_statistics_groups(18, 600, stats_seed))
                .expect("stats groups"),
        );

        let mut hierarchy_model =
            HierarchyChartModel::new(make_hierarchy_tree(hierarchy_seed)).expect("hierarchy tree");
        hierarchy_model.style.mode = config.hierarchy_mode;
        let hierarchy = HierarchyChartHandle::new(hierarchy_model);

        let network = NetworkChartHandle::new(
            make_network_model(config.network_mode, network_seed).expect("network model"),
        );

        let mut polar_model =
            PolarChartModel::new_radar(make_radar_dimensions(), make_radar_series(polar_seed))
                .expect("radar data");
        polar_model.mode = config.polar_mode;
        polar_model.style.mode = config.polar_mode;
        let polar = PolarChartHandle::new(polar_model);

        let gauge_value = seed_range(gauge_seed, 12, 8.0, 96.0);
        let gauge = GaugeChartHandle::new({
            let mut m = GaugeChartModel::new(0.0, 100.0, 0.0).expect("gauge model");
            m.set_value_transition(gauge_value, 0.45);
            m
        });

        let funnel = FunnelChartHandle::new(
            FunnelChartModel::new(make_funnel_stages(funnel_seed)).expect("funnel stages"),
        );

        let geo =
            GeoChartHandle::new(GeoChartModel::new(make_geo_shapes(geo_seed)).expect("geo shapes"));

        let mut models = Self {
            line,
            multi,
            area,
            bar,
            hist,
            scatter,
            candle,
            heat,
            stacked_area,
            density,
            contour,
            stats,
            hierarchy,
            network,
            polar,
            gauge,
            funnel,
            geo,
        };
        models.apply_preset(config.preset);
        models.apply_advanced_options(config);
        models
    }

    fn apply_preset(&mut self, preset: DataPreset) {
        let (
            line_points,
            area_points,
            scatter_points,
            multi_series,
            multi_segments,
            multi_points,
            bar_bins,
            candle_max,
            heat_x,
            heat_y,
            density_x,
            density_y,
            density_points,
            contour_segments,
            contour_levels,
            hierarchy_leaves,
            network_nodes,
            network_links,
            polar_series,
            geo_points,
        ) = match preset {
            DataPreset::Fast => (
                2_500,
                2_500,
                1_600,
                36,
                10_000,
                512,
                2_000,
                4_500,
                80,
                42,
                72,
                36,
                80_000,
                7_000,
                vec![-0.4, 0.0, 0.4],
                600,
                96,
                700,
                6,
                6_000,
            ),
            DataPreset::Balanced => (
                8_000,
                8_000,
                4_000,
                64,
                22_000,
                1_024,
                8_000,
                12_000,
                128,
                64,
                128,
                64,
                180_000,
                20_000,
                vec![-0.6, -0.2, 0.2, 0.6],
                2_000,
                256,
                2_000,
                16,
                20_000,
            ),
            DataPreset::Detail => (
                20_000,
                20_000,
                7_000,
                96,
                40_000,
                1_800,
                18_000,
                20_000,
                160,
                96,
                160,
                88,
                250_000,
                35_000,
                vec![-0.8, -0.4, 0.0, 0.4, 0.8],
                4_000,
                512,
                4_000,
                20,
                35_000,
            ),
        };

        Self::with_locked_model(&self.line.0, |m| {
            m.set_downsample_max_points(line_points);
        });
        Self::with_locked_model(&self.area.0, |m| {
            m.set_downsample_max_points(area_points);
        });
        Self::with_locked_model(&self.scatter.0, |m| {
            m.style.max_points = scatter_points.max(512);
            m.set_max_points(scatter_points);
        });
        Self::with_locked_model(&self.multi.0, |m| {
            m.style.max_series = multi_series;
            m.style.max_total_segments = multi_segments;
            m.style.max_points_per_series = multi_points;
        });
        Self::with_locked_model(&self.bar.0, |m| {
            m.style.max_bins = bar_bins;
        });
        Self::with_locked_model(&self.candle.0, |m| {
            m.style.max_candles = candle_max;
        });
        Self::with_locked_model(&self.heat.0, |m| {
            m.style.max_cells_x = heat_x;
            m.style.max_cells_y = heat_y;
        });
        Self::with_locked_model(&self.density.0, |m| {
            m.style.max_cells_x = density_x;
            m.style.max_cells_y = density_y;
            m.style.max_points = density_points;
        });
        Self::with_locked_model(&self.contour.0, |m| {
            m.style.max_segments = contour_segments;
            m.style.levels = contour_levels;
        });
        Self::with_locked_model(&self.hierarchy.0, |m| {
            m.style.max_leaves = hierarchy_leaves;
        });
        Self::with_locked_model(&self.network.0, |m| {
            m.style.max_nodes = network_nodes;
            m.style.max_links = network_links;
        });
        Self::with_locked_model(&self.polar.0, |m| {
            m.style.max_series = polar_series;
        });
        Self::with_locked_model(&self.geo.0, |m| {
            m.style.max_points = geo_points;
        });
    }

    fn with_locked_model<M, F>(model: &std::sync::Arc<std::sync::Mutex<M>>, apply: F)
    where
        F: FnOnce(&mut M),
    {
        if let Ok(mut locked) = model.lock() {
            apply(&mut locked);
        }
    }

    // Keep per-chart option application explicit for type safety:
    // each model owns a distinct style struct and mutation surface.
    fn apply_advanced_options(&mut self, config: &GalleryConfig) {
        self.apply_advanced_series_options(config);
        self.apply_advanced_field_options(config);
        self.apply_advanced_structural_options(config);
        self.apply_advanced_specialized_options(config);
    }

    fn apply_advanced_series_options(&mut self, cfg: &GalleryConfig) {
        Self::with_locked_model(&self.line.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Line) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.line = p.a;
                m.style.crosshair = p.c;
                m.style.text = p.text;
            }
            m.style.stroke_width = cfg.pick_f32(ChartKind::Line, "stroke", &[1.0, 1.5, 2.2, 3.0]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Line, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min = cfg.pick_f32(ChartKind::Line, "pinch", &[0.01, 0.05, 0.1]);
            m.set_downsample_max_points(cfg.pick_usize(
                ChartKind::Line,
                "max_points",
                &[2_500, 8_000, 20_000, 60_000],
            ));
        });

        Self::with_locked_model(&self.area.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Area) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.line = p.a;
                m.style.area = p.b;
                m.style.crosshair = p.c;
                m.style.text = p.text;
            }
            m.style.stroke_width = cfg.pick_f32(ChartKind::Area, "stroke", &[1.0, 1.5, 2.2, 3.0]);
            m.style.baseline_y = cfg.pick_f32(ChartKind::Area, "baseline", &[-0.5, 0.0, 0.5]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Area, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min = cfg.pick_f32(ChartKind::Area, "pinch", &[0.01, 0.05, 0.1]);
            m.set_downsample_max_points(cfg.pick_usize(
                ChartKind::Area,
                "max_points",
                &[2_500, 8_000, 20_000, 60_000],
            ));
        });

        Self::with_locked_model(&self.multi.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::MultiLine) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.crosshair = p.c;
                m.style.text = p.text;
            }
            m.style.stroke_width =
                cfg.pick_f32(ChartKind::MultiLine, "stroke", &[0.8, 1.2, 1.8, 2.5]);
            m.style.series_alpha =
                cfg.pick_f32(ChartKind::MultiLine, "alpha", &[0.30, 0.45, 0.60, 0.75]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::MultiLine, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min =
                cfg.pick_f32(ChartKind::MultiLine, "pinch", &[0.01, 0.05, 0.1]);
            m.style.max_series =
                cfg.pick_usize(ChartKind::MultiLine, "max_series", &[24, 48, 72, 96]);
            m.style.max_total_segments = cfg.pick_usize(
                ChartKind::MultiLine,
                "max_segments",
                &[8_000, 22_000, 40_000, 80_000],
            );
            m.style.max_points_per_series = cfg.pick_usize(
                ChartKind::MultiLine,
                "max_points_per_series",
                &[256, 512, 1_024, 2_048],
            );
            m.set_gap_dx(cfg.pick_f32(ChartKind::MultiLine, "gap_dx", &[2.0, 6.0, 10.0, 14.0]));
        });

        Self::with_locked_model(&self.bar.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Bar) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.crosshair = p.c;
            }
            m.style.bar_alpha = cfg.pick_f32(ChartKind::Bar, "alpha", &[0.45, 0.65, 0.85, 1.0]);
            m.style.max_series = cfg.pick_usize(ChartKind::Bar, "max_series", &[2, 4, 8, 16]);
            m.style.max_bins =
                cfg.pick_usize(ChartKind::Bar, "max_bins", &[2_000, 8_000, 20_000, 40_000]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Bar, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min = cfg.pick_f32(ChartKind::Bar, "pinch", &[0.01, 0.05, 0.1]);
        });

        Self::with_locked_model(&self.hist.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Histogram) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.bar = p.a;
                m.style.crosshair = p.c;
                m.style.text = p.text;
            }
            m.style.bins = cfg.pick_usize(ChartKind::Histogram, "bins", &[24, 48, 96, 192]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Histogram, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min =
                cfg.pick_f32(ChartKind::Histogram, "pinch", &[0.01, 0.05, 0.1]);
        });

        Self::with_locked_model(&self.scatter.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Scatter) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.points = p.a;
                m.style.crosshair = p.c;
                m.style.text = p.text;
            }
            m.style.point_radius =
                cfg.pick_f32(ChartKind::Scatter, "radius", &[0.8, 1.2, 1.8, 2.5]);
            m.style.hover_hit_radius_px =
                cfg.pick_f32(ChartKind::Scatter, "hit", &[8.0, 12.0, 16.0, 24.0]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Scatter, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min = cfg.pick_f32(ChartKind::Scatter, "pinch", &[0.01, 0.05, 0.1]);
            let max_points = cfg.pick_usize(
                ChartKind::Scatter,
                "max_points",
                &[1_600, 4_000, 7_000, 14_000],
            );
            m.style.max_points = max_points.max(512);
            m.set_max_points(max_points);
        });

        Self::with_locked_model(&self.candle.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Candlestick) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.crosshair = p.c;
                m.style.text = p.text;
                m.style.up = p.a;
                m.style.down = p.b;
                m.style.wick = p.c;
            }
            m.style.stroke_width =
                cfg.pick_f32(ChartKind::Candlestick, "stroke", &[1.0, 1.5, 2.0, 2.8]);
            m.style.max_candles = cfg.pick_usize(
                ChartKind::Candlestick,
                "max_candles",
                &[3_500, 12_000, 20_000, 35_000],
            );
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Candlestick, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min =
                cfg.pick_f32(ChartKind::Candlestick, "pinch", &[0.01, 0.05, 0.1]);
        });
    }

    fn apply_advanced_field_options(&mut self, cfg: &GalleryConfig) {
        Self::with_locked_model(&self.heat.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Heatmap) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
            }
            m.style.max_cells_x =
                cfg.pick_usize(ChartKind::Heatmap, "cells_x", &[64, 128, 192, 256]);
            m.style.max_cells_y = cfg.pick_usize(ChartKind::Heatmap, "cells_y", &[32, 64, 96, 128]);
        });

        Self::with_locked_model(&self.stacked_area.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::StackedArea) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.crosshair = p.c;
            }
            m.style.stroke_width =
                cfg.pick_f32(ChartKind::StackedArea, "stroke", &[0.8, 1.2, 1.8, 2.4]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::StackedArea, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min =
                cfg.pick_f32(ChartKind::StackedArea, "pinch", &[0.01, 0.05, 0.1]);
        });

        Self::with_locked_model(&self.density.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::DensityMap) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
            }
            m.style.max_cells_x =
                cfg.pick_usize(ChartKind::DensityMap, "cells_x", &[72, 128, 192, 256]);
            m.style.max_cells_y =
                cfg.pick_usize(ChartKind::DensityMap, "cells_y", &[36, 64, 96, 128]);
            m.style.max_points = cfg.pick_usize(
                ChartKind::DensityMap,
                "max_points",
                &[80_000, 180_000, 250_000, 400_000],
            );
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::DensityMap, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min =
                cfg.pick_f32(ChartKind::DensityMap, "pinch", &[0.01, 0.05, 0.1]);
        });

        Self::with_locked_model(&self.contour.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Contour) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.stroke = p.a;
            }
            m.style.stroke_width =
                cfg.pick_f32(ChartKind::Contour, "stroke", &[1.0, 1.5, 2.2, 3.0]);
            m.style.max_segments = cfg.pick_usize(
                ChartKind::Contour,
                "max_segments",
                &[7_000, 20_000, 35_000, 60_000],
            );
            let levels = [
                vec![-0.4, 0.0, 0.4],
                vec![-0.6, -0.2, 0.2, 0.6],
                vec![-0.8, -0.4, 0.0, 0.4, 0.8],
                vec![-0.9, -0.6, -0.3, 0.0, 0.3, 0.6, 0.9],
            ];
            m.style.levels =
                levels[cfg.option_index(ChartKind::Contour, "levels", levels.len())].clone();
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Contour, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min = cfg.pick_f32(ChartKind::Contour, "pinch", &[0.01, 0.05, 0.1]);
        });

        Self::with_locked_model(&self.stats.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Statistics) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.accent = p.a;
                m.style.crosshair = p.c;
            }
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Statistics, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min =
                cfg.pick_f32(ChartKind::Statistics, "pinch", &[0.01, 0.05, 0.1]);
        });
    }

    fn apply_advanced_structural_options(&mut self, cfg: &GalleryConfig) {
        Self::with_locked_model(&self.hierarchy.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Hierarchy) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.border = p.a;
            }
            m.style.max_leaves = cfg.pick_usize(
                ChartKind::Hierarchy,
                "max_leaves",
                &[600, 2_000, 4_000, 8_000],
            );
        });

        Self::with_locked_model(&self.network.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Network) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.node = p.a;
                m.style.link = p.b;
            }
            m.style.node_radius =
                cfg.pick_f32(ChartKind::Network, "radius", &[3.0, 6.0, 10.0, 14.0]);
            m.style.max_nodes =
                cfg.pick_usize(ChartKind::Network, "max_nodes", &[96, 256, 512, 1024]);
            m.style.max_links =
                cfg.pick_usize(ChartKind::Network, "max_links", &[700, 2_000, 4_000, 8_000]);
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Network, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min = cfg.pick_f32(ChartKind::Network, "pinch", &[0.01, 0.05, 0.1]);
        });

        Self::with_locked_model(&self.polar.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Polar) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.stroke = p.a;
            }
            m.style.fill_alpha =
                cfg.pick_f32(ChartKind::Polar, "fill_alpha", &[0.10, 0.20, 0.35, 0.50]);
            m.style.max_series = cfg.pick_usize(ChartKind::Polar, "max_series", &[4, 8, 16, 32]);
            let ranges = [0.8f32, 1.0, 1.2, 1.5];
            let range = ranges[cfg.option_index(ChartKind::Polar, "range", ranges.len())];
            m.style.min_value = 0.0;
            m.style.max_value = range;
        });
    }

    fn apply_advanced_specialized_options(&mut self, cfg: &GalleryConfig) {
        Self::with_locked_model(&self.gauge.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Gauge) {
                m.style.bg = p.bg;
                m.style.track = p.grid;
                m.style.fill = p.a;
                m.style.needle = p.c;
                m.style.text = p.text;
            }
            m.style.stroke_width =
                cfg.pick_f32(ChartKind::Gauge, "stroke", &[4.0, 8.0, 12.0, 16.0]);
            let spans = [0.5f32, 0.75, 1.0, 1.25];
            let span = spans[cfg.option_index(ChartKind::Gauge, "span", spans.len())];
            m.style.angle_start_rad = -std::f32::consts::PI * span;
            m.style.angle_end_rad = std::f32::consts::PI * span;
            m.transition_step_sec = cfg.pick_f32(
                ChartKind::Gauge,
                "transition_dt",
                &[1.0 / 120.0, 1.0 / 90.0, 1.0 / 60.0, 1.0 / 30.0],
            );
        });

        Self::with_locked_model(&self.funnel.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Funnel) {
                m.style.bg = p.bg;
                m.style.text = p.text;
                m.style.fill = p.b;
                m.style.stroke = p.c;
            }
        });

        Self::with_locked_model(&self.geo.0, |m| {
            if let Some(p) = cfg.theme_palette(ChartKind::Geo) {
                m.style.bg = p.bg;
                m.style.grid = p.grid;
                m.style.text = p.text;
                m.style.stroke = p.a;
            }
            m.style.stroke_width = cfg.pick_f32(ChartKind::Geo, "stroke", &[0.8, 1.2, 1.8, 2.6]);
            m.style.max_points = cfg.pick_usize(
                ChartKind::Geo,
                "max_points",
                &[6_000, 20_000, 35_000, 60_000],
            );
            m.style.scroll_zoom_factor =
                cfg.pick_f32(ChartKind::Geo, "scroll", &[0.01, 0.02, 0.04]);
            m.style.pinch_zoom_min = cfg.pick_f32(ChartKind::Geo, "pinch", &[0.01, 0.05, 0.1]);
        });
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let (mut width, mut height) = (1200u32, 840u32);
    if let Ok(v) = std::env::var("BLINC_WINDOW_SIZE") {
        if let Some((w, h)) = v.split_once('x') {
            if let (Ok(w), Ok(h)) = (w.trim().parse::<u32>(), h.trim().parse::<u32>()) {
                if w >= 320 && h >= 240 {
                    width = w;
                    height = h;
                }
            }
        }
    }

    let config = WindowConfig {
        title: "blinc_charts: Gallery".to_string(),
        width,
        height,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    if std::env::var_os("BLINC_DEBUG_GALLERY_CTX").is_some() {
        tracing::info!(
            "charts_gallery_demo: ctx.width={:.1} ctx.height={:.1} scale_factor={:.3}",
            ctx.width,
            ctx.height,
            ctx.scale_factor
        );
    }
    if std::env::var_os("BLINC_DEBUG_GALLERY_BOUNDS").is_some() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static REGISTERED: AtomicBool = AtomicBool::new(false);
        if !REGISTERED.swap(true, Ordering::Relaxed) {
            ctx.query("charts_gallery_tabs").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: tabs bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_main").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: main bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_canvas").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: canvas bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar_scroll").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar_scroll bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar_item_0").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar_item_0 bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar_item_10").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar_item_10 bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
        }
    }

    let line_n = std::env::var("BLINC_CHARTS_N")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v >= 2)
        .unwrap_or(200_000);

    let initial_selected = parse_initial_selected(
        std::env::var("BLINC_GALLERY_SELECTED").ok().as_deref(),
        ITEMS.len(),
    );
    let selected = ctx.use_state_keyed("charts_gallery_selected", move || initial_selected);

    let config = ctx.use_state_keyed("charts_gallery_config", move || {
        GalleryConfig::new(line_n, ITEMS.len())
    });

    let models = {
        let initial_cfg = config.try_get().expect("gallery config exists");
        ctx.use_state_keyed("charts_gallery_models", move || {
            GalleryModels::new(&initial_cfg)
        })
    };

    let layout = layout_mode(ctx.width, ctx.height);
    let dense = ctx.height < 460.0;

    let header = {
        let config_for_header = config.clone();
        stateful::<NoState>()
            .deps([config.signal_id()])
            .on_state(move |_ctx| {
                let cfg = config_for_header.try_get().expect("config exists");
                div()
                    .flex_row()
                    .items_end()
                    .justify_between()
                    .child(
                        text("blinc_charts gallery")
                            .size(if dense { 18.0 } else { 24.0 })
                            .weight(FontWeight::Bold)
                            .color(Color::rgba(0.95, 0.96, 0.98, 1.0))
                            .no_wrap(),
                    )
                    .child(
                        text(format!(
                            "N={} | preset={} | noise={} | seed=0x{:08X}",
                            cfg.line_n,
                            cfg.preset.label(),
                            if cfg.noise_enabled { "on" } else { "off" },
                            cfg.base_seed as u32
                        ))
                        .size(if dense { 10.0 } else { 12.0 })
                        .color(Color::rgba(0.70, 0.75, 0.82, 1.0))
                        .no_wrap(),
                    )
            })
    };

    let sidebar = {
        let selected_for_list = selected.clone();
        let selected_signal = selected.signal_id();

        div()
            .id("charts_gallery_sidebar")
            .w(300.0)
            .h_full()
            .rounded(14.0)
            .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06))
            .p(if dense { 4.0 } else { 10.0 })
            .flex_col()
            .gap(if dense { 3.0 } else { 8.0 })
            .child(
                text("Charts")
                    .size(14.0)
                    .weight(FontWeight::SemiBold)
                    .color(Color::rgba(1.0, 1.0, 1.0, 0.80)),
            )
            .child(div().h(1.0).bg(Color::rgba(1.0, 1.0, 1.0, 0.06)))
            .child({
                let mut list = div().flex_col().gap(6.0);
                let mut last_group: Option<&'static str> = None;

                for (i, item) in ITEMS.iter().enumerate() {
                    let item = *item;

                    if last_group != Some(item.group) {
                        last_group = Some(item.group);
                        list = list.child(
                            text(item.group)
                                .size(10.0)
                                .weight(FontWeight::SemiBold)
                                .color(Color::rgba(0.58, 0.66, 0.76, 0.95)),
                        );
                    }

                    let title = item.title;
                    let item_id = format!("charts_gallery_sidebar_item_{i}");
                    let selected_for_state = selected_for_list.clone();
                    let selected_for_click = selected_for_list.clone();

                    list = list.child(
                        stateful::<ButtonState>()
                            .initial(ButtonState::Idle)
                            .deps([selected_signal])
                            .on_state(move |ctx| {
                                let is_selected = selected_for_state.get() == i;
                                let bg = if is_selected {
                                    Color::rgba(0.20, 0.35, 0.60, 0.55)
                                } else {
                                    match ctx.state() {
                                        ButtonState::Idle => Color::rgba(1.0, 1.0, 1.0, 0.04),
                                        ButtonState::Hovered => Color::rgba(1.0, 1.0, 1.0, 0.07),
                                        ButtonState::Pressed => Color::rgba(1.0, 1.0, 1.0, 0.10),
                                        ButtonState::Disabled => Color::rgba(1.0, 1.0, 1.0, 0.02),
                                    }
                                };
                                let fg = if is_selected {
                                    Color::rgba(0.95, 0.96, 0.98, 1.0)
                                } else {
                                    Color::rgba(0.85, 0.88, 0.92, 0.90)
                                };
                                div()
                                    .id(item_id.clone())
                                    .px(10.0)
                                    .py(8.0)
                                    .rounded(10.0)
                                    .bg(bg)
                                    .child(
                                        text(title)
                                            .size(12.0)
                                            .weight(FontWeight::Medium)
                                            .color(fg)
                                            .no_wrap()
                                            .pointer_events_none(),
                                    )
                            })
                            .on_click(move |_| {
                                selected_for_click.set(i);
                            }),
                    );
                }

                div()
                    .flex_1()
                    .min_h(0.0)
                    .w_full()
                    .id("charts_gallery_sidebar_scroll")
                    .overflow_y_scroll()
                    .child(list)
            })
    };

    let tabs = {
        let selected_for_list = selected.clone();
        let selected_signal = selected.signal_id();

        let compact_tabs = dense;

        let mut list = div()
            .id("charts_gallery_tabs")
            .w_full()
            .flex_row()
            .gap(if dense { 2.0 } else { 8.0 })
            .p(if dense { 2.0 } else { 8.0 })
            .rounded(14.0)
            .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06));

        if compact_tabs {
            list = list.max_h(52.0).overflow_x_scroll();
        } else {
            list = list.max_h(140.0).overflow_y_scroll().flex_wrap();
        }

        for (i, item) in ITEMS.iter().enumerate() {
            let title = item.title;
            let item_id = format!("charts_gallery_tab_item_{i}");
            let selected_for_state = selected_for_list.clone();
            let selected_for_click = selected_for_list.clone();

            list = list.child(
                stateful::<ButtonState>()
                    .initial(ButtonState::Idle)
                    .deps([selected_signal])
                    .on_state(move |ctx| {
                        let is_selected = selected_for_state.get() == i;
                        let bg = if is_selected {
                            Color::rgba(0.20, 0.35, 0.60, 0.55)
                        } else {
                            match ctx.state() {
                                ButtonState::Idle => Color::rgba(1.0, 1.0, 1.0, 0.04),
                                ButtonState::Hovered => Color::rgba(1.0, 1.0, 1.0, 0.07),
                                ButtonState::Pressed => Color::rgba(1.0, 1.0, 1.0, 0.10),
                                ButtonState::Disabled => Color::rgba(1.0, 1.0, 1.0, 0.02),
                            }
                        };
                        let fg = if is_selected {
                            Color::rgba(0.95, 0.96, 0.98, 1.0)
                        } else {
                            Color::rgba(0.85, 0.88, 0.92, 0.90)
                        };
                        div()
                            .id(item_id.clone())
                            .px(if dense { 4.0 } else { 10.0 })
                            .py(if dense { 3.0 } else { 8.0 })
                            .rounded(999.0)
                            .bg(bg)
                            .child(
                                text(title)
                                    .size(12.0)
                                    .weight(FontWeight::Medium)
                                    .color(fg)
                                    .no_wrap()
                                    .pointer_events_none(),
                            )
                    })
                    .on_click(move |_| {
                        selected_for_click.set(i);
                    }),
            );
        }

        list
    };

    let main = {
        let selected_for_main = selected.clone();
        let selected_signal = selected.signal_id();

        let config_for_main = config.clone();
        let config_signal = config.signal_id();

        let models_for_main = models.clone();
        let models_signal = models.signal_id();

        stateful::<NoState>()
            .deps([selected_signal, config_signal, models_signal])
            .on_state(move |_ctx| {
                let idx = selected_for_main.get();
                let item = ITEMS.get(idx).copied().unwrap_or_else(|| ITEMS[0]);

                let cfg = config_for_main.try_get().expect("config exists");
                let m = models_for_main.try_get().expect("models exist");

                div()
                    .id("charts_gallery_main")
                    .flex_1()
                    .h_full()
                    .rounded(14.0)
                    .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
                    .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06))
                    .p(if dense { 4.0 } else { 10.0 })
                    .flex_col()
                    .gap(if dense { 3.0 } else { 10.0 })
                    .child(
                        div()
                            .flex_col()
                            .gap(2.0)
                            .child(
                                div()
                                    .flex_row()
                                    .justify_between()
                                    .child(
                                        text(item.title)
                                            .size(16.0)
                                            .weight(FontWeight::SemiBold)
                                            .color(Color::rgba(0.92, 0.93, 0.95, 1.0)),
                                    )
                                    .child(
                                        text(item.group)
                                            .size(10.0)
                                            .weight(FontWeight::SemiBold)
                                            .color(Color::rgba(0.57, 0.66, 0.78, 0.95)),
                                    ),
                            )
                            .child(
                                text(item.subtitle)
                                    .size(12.0)
                                    .color(Color::rgba(0.68, 0.72, 0.78, 1.0)),
                            )
                            .child(
                                text(format!(
                                    "seed 0x{:08X} | preset {} | noise {}",
                                    cfg.seed_for_index(idx) as u32,
                                    cfg.preset.label(),
                                    if cfg.noise_enabled { "on" } else { "off" }
                                ))
                                .size(10.0)
                                .color(Color::rgba(0.55, 0.60, 0.67, 1.0)),
                            ),
                    )
                    .child(control_panel(
                        idx,
                        item,
                        dense,
                        config_for_main.clone(),
                        models_for_main.clone(),
                    ))
                    .child(
                        div()
                            .id("charts_gallery_canvas")
                            .flex_1()
                            .rounded(14.0)
                            .overflow_clip()
                            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                            .child_box(chart_for(item.kind, m)),
                    )
            })
    };

    let root = div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .p(if dense { 3.0 } else { 12.0 })
        .flex_col()
        .gap(if dense { 3.0 } else { 12.0 })
        .child(header);

    match layout {
        LayoutMode::Wide => root.child(
            div()
                .flex_1()
                .flex_row()
                .gap(12.0)
                .child(sidebar)
                .child(main),
        ),
        LayoutMode::Narrow => root.child(tabs).child(div().flex_1().child(main)),
    }
}

fn control_panel(
    idx: usize,
    item: ChartEntry,
    dense: bool,
    config: State<GalleryConfig>,
    models: State<GalleryModels>,
) -> impl ElementBuilder {
    let cfg = config.try_get().expect("config exists");

    let mut row = div()
        .flex_row()
        .flex_wrap()
        .gap(if dense { 4.0 } else { 6.0 })
        .items_center();

    {
        let config_for_click = config.clone();
        let models_for_click = models.clone();
        row = row.child(action_chip("Randomize current", dense, false, move |_| {
            config_for_click.update(|mut c| {
                c.randomize_chart(idx);
                c
            });
            refresh_models(&config_for_click, &models_for_click);
        }));
    }

    {
        let config_for_click = config.clone();
        let models_for_click = models.clone();
        row = row.child(action_chip("Randomize all", dense, false, move |_| {
            config_for_click.update(|mut c| {
                c.randomize_all();
                c
            });
            refresh_models(&config_for_click, &models_for_click);
        }));
    }

    {
        let config_for_click = config.clone();
        let models_for_click = models.clone();
        row = row.child(action_chip(
            format!("Preset: {}", cfg.preset.label()),
            dense,
            true,
            move |_| {
                config_for_click.update(|mut c| {
                    c.preset = c.preset.next();
                    c
                });
                refresh_models(&config_for_click, &models_for_click);
            },
        ));
    }

    {
        let config_for_click = config.clone();
        let models_for_click = models.clone();
        row = row.child(action_chip(
            format!("Noise: {}", if cfg.noise_enabled { "On" } else { "Off" }),
            dense,
            cfg.noise_enabled,
            move |_| {
                config_for_click.update(|mut c| {
                    c.noise_enabled = !c.noise_enabled;
                    c
                });
                refresh_models(&config_for_click, &models_for_click);
            },
        ));
    }

    {
        let config_for_click = config.clone();
        let models_for_click = models.clone();
        row = row.child(action_chip("Reset view/style", dense, false, move |_| {
            config_for_click.update(|mut c| {
                c.reset();
                c
            });
            refresh_models(&config_for_click, &models_for_click);
        }));
    }

    row = row.child(cycle_index_chip(
        &cfg, dense, &config, &models, item.kind, "theme", "Theme: T", 4, 1,
    ));

    match item.kind {
        ChartKind::Line => {
            let kind = ChartKind::Line;
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "stroke",
                "Stroke",
                &[1.0, 1.5, 2.2, 3.0],
                1,
            ));
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "max_points",
                "MaxPts",
                &[2_500, 8_000, 20_000, 60_000],
            ));
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::Area => {
            let kind = ChartKind::Area;
            for (name, label, values) in [
                ("stroke", "Stroke", &[1.0, 1.5, 2.2, 3.0][..]),
                ("baseline", "Baseline", &[-0.5, 0.0, 0.5][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 1,
                ));
            }
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "max_points",
                "MaxPts",
                &[2_500, 8_000, 20_000, 60_000],
            ));
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::MultiLine => {
            let kind = ChartKind::MultiLine;
            for (name, label, values, decimals) in [
                ("stroke", "Stroke", &[0.8, 1.2, 1.8, 2.5][..], 1usize),
                ("alpha", "Alpha", &[0.30, 0.45, 0.60, 0.75][..], 2usize),
                ("gap_dx", "Gap", &[2.0, 6.0, 10.0, 14.0][..], 1usize),
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..], 2usize),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..], 2usize),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, decimals,
                ));
            }
            for (name, label, values) in [
                ("max_series", "Series", &[24, 48, 72, 96][..]),
                ("max_segments", "Seg", &[8_000, 22_000, 40_000, 80_000][..]),
                (
                    "max_points_per_series",
                    "Pts/Series",
                    &[256, 512, 1_024, 2_048][..],
                ),
            ] {
                row = row.child(cycle_usize_chip(
                    &cfg, dense, &config, &models, kind, name, label, values,
                ));
            }
        }
        ChartKind::Bar => {
            let config_for_click = config.clone();
            let models_for_click = models.clone();
            row = row.child(action_chip(
                format!(
                    "Bars: {}",
                    if cfg.bar_stacked {
                        "Stacked"
                    } else {
                        "Grouped"
                    }
                ),
                dense,
                true,
                move |_| {
                    config_for_click.update(|mut c| {
                        c.bar_stacked = !c.bar_stacked;
                        c
                    });
                    refresh_models(&config_for_click, &models_for_click);
                },
            ));
            let kind = ChartKind::Bar;
            for (name, label, values, decimals) in [
                ("alpha", "Alpha", &[0.45, 0.65, 0.85, 1.0][..], 2usize),
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..], 2usize),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..], 2usize),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, decimals,
                ));
            }
            for (name, label, values) in [
                ("max_series", "Series", &[2, 4, 8, 16][..]),
                ("max_bins", "Bins", &[2_000, 8_000, 20_000, 40_000][..]),
            ] {
                row = row.child(cycle_usize_chip(
                    &cfg, dense, &config, &models, kind, name, label, values,
                ));
            }
        }
        ChartKind::Histogram => {
            let kind = ChartKind::Histogram;
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "bins",
                "Bins",
                &[24, 48, 96, 192],
            ));
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::Scatter => {
            let kind = ChartKind::Scatter;
            for (name, label, values, decimals) in [
                ("radius", "Radius", &[0.8, 1.2, 1.8, 2.5][..], 1usize),
                ("hit", "Hit", &[8.0, 12.0, 16.0, 24.0][..], 0usize),
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..], 2usize),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..], 2usize),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, decimals,
                ));
            }
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "max_points",
                "MaxPts",
                &[1_600, 4_000, 7_000, 14_000],
            ));
        }
        ChartKind::Candlestick => {
            let kind = ChartKind::Candlestick;
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "stroke",
                "Stroke",
                &[1.0, 1.5, 2.0, 2.8],
                1,
            ));
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "max_candles",
                "MaxCandle",
                &[3_500, 12_000, 20_000, 35_000],
            ));
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::Heatmap => {
            let kind = ChartKind::Heatmap;
            for (name, label, values) in [
                ("cells_x", "CellsX", &[64, 128, 192, 256][..]),
                ("cells_y", "CellsY", &[32, 64, 96, 128][..]),
            ] {
                row = row.child(cycle_usize_chip(
                    &cfg, dense, &config, &models, kind, name, label, values,
                ));
            }
        }
        ChartKind::StackedArea => {
            let config_for_click = config.clone();
            let models_for_click = models.clone();
            row = row.child(action_chip(
                format!("Mode: {}", stacked_mode_label(cfg.stacked_mode)),
                dense,
                true,
                move |_| {
                    config_for_click.update(|mut c| {
                        c.stacked_mode = next_stacked_mode(c.stacked_mode);
                        c
                    });
                    refresh_models(&config_for_click, &models_for_click);
                },
            ));
            let kind = ChartKind::StackedArea;
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "stroke",
                "Stroke",
                &[0.8, 1.2, 1.8, 2.4],
                1,
            ));
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::DensityMap => {
            let kind = ChartKind::DensityMap;
            for (name, label, values) in [
                ("cells_x", "CellsX", &[72, 128, 192, 256][..]),
                ("cells_y", "CellsY", &[36, 64, 96, 128][..]),
                (
                    "max_points",
                    "MaxPts",
                    &[80_000, 180_000, 250_000, 400_000][..],
                ),
            ] {
                row = row.child(cycle_usize_chip(
                    &cfg, dense, &config, &models, kind, name, label, values,
                ));
            }
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::Contour => {
            let kind = ChartKind::Contour;
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "stroke",
                "Stroke",
                &[1.0, 1.5, 2.2, 3.0],
                1,
            ));
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "max_segments",
                "Segments",
                &[7_000, 20_000, 35_000, 60_000],
            ));
            row = row.child(cycle_index_chip(
                &cfg, dense, &config, &models, kind, "levels", "Levels L", 4, 1,
            ));
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::Statistics => {
            let kind = ChartKind::Statistics;
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::Hierarchy => {
            let config_for_click = config.clone();
            let models_for_click = models.clone();
            row = row.child(action_chip(
                format!("Layout: {}", hierarchy_mode_label(cfg.hierarchy_mode)),
                dense,
                true,
                move |_| {
                    config_for_click.update(|mut c| {
                        c.hierarchy_mode = next_hierarchy_mode(c.hierarchy_mode);
                        c
                    });
                    refresh_models(&config_for_click, &models_for_click);
                },
            ));
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                ChartKind::Hierarchy,
                "max_leaves",
                "Leaves",
                &[600, 2_000, 4_000, 8_000],
            ));
        }
        ChartKind::Network => {
            let config_for_click = config.clone();
            let models_for_click = models.clone();
            row = row.child(action_chip(
                format!("Network: {}", network_mode_label(cfg.network_mode)),
                dense,
                true,
                move |_| {
                    config_for_click.update(|mut c| {
                        c.network_mode = next_network_mode(c.network_mode);
                        c
                    });
                    refresh_models(&config_for_click, &models_for_click);
                },
            ));
            let kind = ChartKind::Network;
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "radius",
                "Radius",
                &[3.0, 6.0, 10.0, 14.0],
                1,
            ));
            for (name, label, values) in [
                ("max_nodes", "Nodes", &[96, 256, 512, 1_024][..]),
                ("max_links", "Links", &[700, 2_000, 4_000, 8_000][..]),
            ] {
                row = row.child(cycle_usize_chip(
                    &cfg, dense, &config, &models, kind, name, label, values,
                ));
            }
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        ChartKind::Polar => {
            let config_for_click = config.clone();
            let models_for_click = models.clone();
            row = row.child(action_chip(
                format!("Polar: {}", polar_mode_label(cfg.polar_mode)),
                dense,
                true,
                move |_| {
                    config_for_click.update(|mut c| {
                        c.polar_mode = next_polar_mode(c.polar_mode);
                        c
                    });
                    refresh_models(&config_for_click, &models_for_click);
                },
            ));
            let kind = ChartKind::Polar;
            for (name, label, values, decimals) in [
                ("fill_alpha", "Fill", &[0.10, 0.20, 0.35, 0.50][..], 2usize),
                ("range", "Range", &[0.8, 1.0, 1.2, 1.5][..], 1usize),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, decimals,
                ));
            }
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "max_series",
                "Series",
                &[4, 8, 16, 32],
            ));
        }
        ChartKind::Gauge => {
            let kind = ChartKind::Gauge;
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "stroke",
                "Stroke",
                &[4.0, 8.0, 12.0, 16.0],
                0,
            ));
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "span",
                "Arc x",
                &[0.5, 0.75, 1.0, 1.25],
                2,
            ));
            row = row.child(cycle_hz_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "transition_dt",
                "Anim",
                &[1.0 / 120.0, 1.0 / 90.0, 1.0 / 60.0, 1.0 / 30.0],
            ));
        }
        ChartKind::Geo => {
            let kind = ChartKind::Geo;
            row = row.child(cycle_f32_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "stroke",
                "Stroke",
                &[0.8, 1.2, 1.8, 2.6],
                1,
            ));
            row = row.child(cycle_usize_chip(
                &cfg,
                dense,
                &config,
                &models,
                kind,
                "max_points",
                "MaxPts",
                &[6_000, 20_000, 35_000, 60_000],
            ));
            for (name, label, values) in [
                ("scroll", "Scroll", &[0.01, 0.02, 0.04][..]),
                ("pinch", "Pinch", &[0.01, 0.05, 0.1][..]),
            ] {
                row = row.child(cycle_f32_chip(
                    &cfg, dense, &config, &models, kind, name, label, values, 2,
                ));
            }
        }
        _ => {}
    }

    div()
        .rounded(10.0)
        .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06))
        .bg(Color::rgba(0.08, 0.09, 0.11, 0.68))
        .p(if dense { 6.0 } else { 8.0 })
        .flex_col()
        .gap(if dense { 4.0 } else { 6.0 })
        .child(
            text(format!("Use case: {}", item.usage))
                .size(11.0)
                .color(Color::rgba(0.76, 0.81, 0.88, 0.95)),
        )
        .child(
            text(format!("Tip: {}", item.controls))
                .size(10.0)
                .color(Color::rgba(0.62, 0.69, 0.77, 0.95)),
        )
        .child(row)
}

fn cycle_f32_chip(
    cfg: &GalleryConfig,
    dense: bool,
    config: &State<GalleryConfig>,
    models: &State<GalleryModels>,
    kind: ChartKind,
    name: &'static str,
    label: &'static str,
    values: &[f32],
    decimals: usize,
) -> impl ElementBuilder {
    let value = cfg.pick_f32(kind, name, values);
    let chip_label = format!("{label} {value:.decimals$}");
    cycle_option_chip(
        chip_label,
        dense,
        config.clone(),
        models.clone(),
        kind,
        name,
        values.len(),
    )
}

fn cycle_usize_chip(
    cfg: &GalleryConfig,
    dense: bool,
    config: &State<GalleryConfig>,
    models: &State<GalleryModels>,
    kind: ChartKind,
    name: &'static str,
    label: &'static str,
    values: &[usize],
) -> impl ElementBuilder {
    let value = cfg.pick_usize(kind, name, values);
    cycle_option_chip(
        format!("{label} {value}"),
        dense,
        config.clone(),
        models.clone(),
        kind,
        name,
        values.len(),
    )
}

fn cycle_index_chip(
    cfg: &GalleryConfig,
    dense: bool,
    config: &State<GalleryConfig>,
    models: &State<GalleryModels>,
    kind: ChartKind,
    name: &'static str,
    label_prefix: &'static str,
    len: usize,
    offset: usize,
) -> impl ElementBuilder {
    let label_value = cfg.option_index(kind, name, len) + offset;
    cycle_option_chip(
        format!("{label_prefix}{label_value}"),
        dense,
        config.clone(),
        models.clone(),
        kind,
        name,
        len,
    )
}

fn cycle_hz_chip(
    cfg: &GalleryConfig,
    dense: bool,
    config: &State<GalleryConfig>,
    models: &State<GalleryModels>,
    kind: ChartKind,
    name: &'static str,
    label: &'static str,
    values: &[f32],
) -> impl ElementBuilder {
    let hz = (1.0 / cfg.pick_f32(kind, name, values)).round() as i32;
    cycle_option_chip(
        format!("{label} {hz}Hz"),
        dense,
        config.clone(),
        models.clone(),
        kind,
        name,
        values.len(),
    )
}

fn action_chip<F>(
    label: impl Into<String>,
    dense: bool,
    active: bool,
    on_click: F,
) -> impl ElementBuilder
where
    F: Fn(&blinc_layout::event_handler::EventContext) + Send + Sync + 'static,
{
    let label = label.into();
    stateful::<ButtonState>()
        .initial(ButtonState::Idle)
        .on_state(move |ctx| {
            let bg = if active {
                Color::rgba(0.20, 0.35, 0.60, 0.50)
            } else {
                match ctx.state() {
                    ButtonState::Idle => Color::rgba(1.0, 1.0, 1.0, 0.03),
                    ButtonState::Hovered => Color::rgba(1.0, 1.0, 1.0, 0.07),
                    ButtonState::Pressed => Color::rgba(1.0, 1.0, 1.0, 0.10),
                    ButtonState::Disabled => Color::rgba(1.0, 1.0, 1.0, 0.02),
                }
            };
            let border = if active {
                Color::rgba(0.55, 0.75, 1.0, 0.45)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.08)
            };
            div()
                .px(if dense { 7.0 } else { 10.0 })
                .py(if dense { 4.0 } else { 6.0 })
                .rounded(999.0)
                .bg(bg)
                .border(1.0, border)
                .child(
                    text(label.clone())
                        .size(if dense { 10.0 } else { 11.0 })
                        .weight(FontWeight::Medium)
                        .color(Color::rgba(0.89, 0.92, 0.96, 1.0))
                        .no_wrap()
                        .pointer_events_none(),
                )
        })
        .on_click(on_click)
}

fn cycle_option_chip(
    label: impl Into<String>,
    dense: bool,
    config: State<GalleryConfig>,
    models: State<GalleryModels>,
    kind: ChartKind,
    name: &'static str,
    len: usize,
) -> impl ElementBuilder {
    action_chip(label, dense, true, move |_| {
        config.update(|mut c| {
            c.cycle_option(kind, name, len);
            c
        });
        refresh_models(&config, &models);
    })
}

fn refresh_models(config: &State<GalleryConfig>, models: &State<GalleryModels>) {
    if let Some(cfg) = config.try_get() {
        models.set(GalleryModels::new(&cfg));
    }
}

fn chart_for(kind: ChartKind, models: GalleryModels) -> Box<dyn ElementBuilder> {
    match kind {
        ChartKind::Line => Box::new(line_chart(models.line)),
        ChartKind::MultiLine => Box::new(multi_line_chart(models.multi)),
        ChartKind::Area => Box::new(area_chart(models.area)),
        ChartKind::Bar => Box::new(bar_chart(models.bar)),
        ChartKind::Histogram => Box::new(histogram_chart(models.hist)),
        ChartKind::Scatter => Box::new(scatter_chart(models.scatter)),
        ChartKind::Candlestick => Box::new(candlestick_chart(models.candle)),
        ChartKind::Heatmap => Box::new(heatmap_chart(models.heat)),
        ChartKind::StackedArea => Box::new(stacked_area_chart(models.stacked_area)),
        ChartKind::DensityMap => Box::new(density_map_chart(models.density)),
        ChartKind::Contour => Box::new(contour_chart(models.contour)),
        ChartKind::Statistics => Box::new(statistics_chart(models.stats)),
        ChartKind::Hierarchy => Box::new(hierarchy_chart(models.hierarchy)),
        ChartKind::Network => Box::new(network_chart(models.network)),
        ChartKind::Polar => Box::new(polar_chart(models.polar)),
        ChartKind::Gauge => Box::new(gauge_chart(models.gauge)),
        ChartKind::Funnel => Box::new(funnel_chart(models.funnel)),
        ChartKind::Geo => Box::new(geo_chart(models.geo)),
    }
}

fn parse_env_seed() -> Option<u64> {
    let raw = std::env::var("BLINC_GALLERY_SEED").ok()?;
    let v = raw.trim();
    if let Some(hex) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        v.parse::<u64>().ok()
    }
}

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

static GALLERY_NOISE_ENABLED: AtomicBool = AtomicBool::new(true);

fn set_gallery_noise_enabled(enabled: bool) {
    GALLERY_NOISE_ENABLED.store(enabled, Ordering::Relaxed);
}

fn gallery_noise_enabled() -> bool {
    GALLERY_NOISE_ENABLED.load(Ordering::Relaxed)
}

fn noise_amount() -> f32 {
    if gallery_noise_enabled() {
        1.0
    } else {
        0.0
    }
}

fn seed01(seed: u64, stream: u64) -> f32 {
    let bits = (splitmix64(seed ^ stream.wrapping_mul(0x9E37_79B9_7F4A_7C15)) >> 40) as u32;
    bits as f32 / 16_777_215.0
}

fn seed_range(seed: u64, stream: u64, lo: f32, hi: f32) -> f32 {
    lo + (hi - lo) * seed01(seed, stream)
}

fn seed_signed(seed: u64, stream: u64, magnitude: f32) -> f32 {
    if !gallery_noise_enabled() {
        return 0.0;
    }
    (seed01(seed, stream) * 2.0 - 1.0) * magnitude
}

fn make_series(n: usize, seed: u64) -> anyhow::Result<TimeSeriesF32> {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);

    let phase = seed_range(seed, 1, 0.0, std::f32::consts::TAU);
    let f0 = seed_range(seed, 2, 0.82, 1.35);
    let f1 = seed_range(seed, 3, 0.08, 0.22);
    let noise = seed_range(seed, 4, 0.02, 0.07);

    for i in 0..n {
        let t = i as f32 * 0.001;
        let saw = ((t * seed_range(seed, 5, 0.9, 1.3) + phase).fract() - 0.5) * noise;
        let v = (t * f0 + phase).sin() * 0.8
            + (t * f1 + phase * 0.55).sin() * 0.2
            + saw * noise_amount();
        x.push(i as f32);
        y.push(v);
    }
    TimeSeriesF32::new(x, y)
}

fn make_multi_series(series_n: usize, points_n: usize, seed: u64) -> Vec<TimeSeriesF32> {
    let mut out = Vec::with_capacity(series_n);
    let gap_every = seed_range(seed, 10, 28.0, 46.0).round() as usize;
    let gap_jump = seed_range(seed, 11, 7.0, 13.0);

    for s in 0..series_n {
        let mut x = Vec::with_capacity(points_n);
        let mut y = Vec::with_capacity(points_n);

        let phase = s as f32 * seed_range(seed, 12, 0.22, 0.51) + seed_range(seed, 13, 0.0, 2.0);
        let f0 = seed_range(seed, 14 + s as u64, 0.04, 0.09);
        let f1 = seed_range(seed, 400 + s as u64, 0.10, 0.24);

        let mut cur_x = 0.0f32;
        for i in 0..points_n {
            cur_x += if i % gap_every.max(3) == 0 && i != 0 {
                gap_jump
            } else {
                1.0
            };
            x.push(cur_x);

            let t = i as f32;
            let vv = (t * f0 + phase).sin() * 0.72 + (t * f1 + phase * 1.8).sin() * 0.24;
            y.push(vv + seed_signed(seed, 800 + i as u64 + (s as u64 * 17), 0.015));
        }
        out.push(TimeSeriesF32::new(x, y).expect("sorted by construction"));
    }
    out
}

fn make_bar_series(series_n: usize, n: usize, seed: u64) -> Vec<TimeSeriesF32> {
    let mut out = Vec::with_capacity(series_n);
    for s in 0..series_n {
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);

        let phase = s as f32 * seed_range(seed, 20, 0.5, 1.2);
        let f0 = seed_range(seed, 21 + s as u64, 0.006, 0.018);
        let f1 = seed_range(seed, 22 + s as u64, 0.022, 0.041);
        let floor = seed_range(seed, 23, 0.04, 0.22);

        for i in 0..n {
            let t = i as f32;
            let raw = (t * f0 + phase).sin() * 0.44 + (t * f1 + phase * 0.7).cos() * 0.22 + 0.58;
            x.push(i as f32);
            y.push(raw.max(floor));
        }
        out.push(TimeSeriesF32::new(x, y).expect("sorted by construction"));
    }
    out
}

fn make_hist_values(n: usize, seed: u64) -> Vec<f32> {
    let mut out = Vec::with_capacity(n);
    let phase = seed_range(seed, 30, 0.0, std::f32::consts::TAU);
    let f0 = seed_range(seed, 31, 1.2, 2.3);
    let f1 = seed_range(seed, 32, 0.6, 1.4);

    for i in 0..n {
        let t = i as f32 * 0.001;
        let a = (t * f0 + phase).sin() * 0.62 + (t * 0.11 + phase * 0.5).sin() * 0.14;
        let b = (t * f1 + phase * 1.8).cos() * 0.36;
        out.push(a + b + seed_signed(seed, 2000 + i as u64, 0.025));
    }
    out
}

fn make_candles(n: usize, seed: u64) -> Vec<Candle> {
    let mut out = Vec::with_capacity(n);
    let mut last = seed_signed(seed, 40, 0.35);

    let drift_scale = seed_range(seed, 41, 0.012, 0.034);
    let noise_scale = seed_range(seed, 42, 0.018, 0.06);

    for i in 0..n {
        let t = i as f32 * 0.01;
        let drift = (t * seed_range(seed, 43, 0.12, 0.28)).sin() * drift_scale;
        let noise = (t * seed_range(seed, 44, 1.2, 2.4)).sin() * noise_scale
            + (t * seed_range(seed, 45, 1.5, 2.8)).cos() * (noise_scale * 0.35);

        let close = last + drift + noise * noise_amount();
        let open = last;
        let wick_amp = seed_range(seed, 46, 0.02, 0.07);
        let hi =
            open.max(close) + (t * 0.7 + seed_range(seed, 47, 0.0, 4.0)).sin().abs() * wick_amp;
        let lo =
            open.min(close) - (t * 0.9 + seed_range(seed, 48, 0.0, 4.0)).cos().abs() * wick_amp;

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

fn make_heat_values(w: usize, h: usize, seed: u64) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let cx = (w as f32) * seed_range(seed, 50, 0.38, 0.66);
    let cy = (h as f32) * seed_range(seed, 51, 0.30, 0.72);
    let spread = seed_range(seed, 52, 24.0, 46.0);
    let wx = seed_range(seed, 53, 9.0, 19.0);
    let wy = seed_range(seed, 54, 8.0, 16.0);

    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - cx) / (w as f32);
            let dy = (y as f32 - cy) / (h as f32);
            let r2 = dx * dx + dy * dy;
            let v = (-r2 * spread).exp()
                + (dx * wx + seed_range(seed, 55, 0.0, 3.0)).sin()
                    * (dy * wy + seed_range(seed, 56, 0.0, 3.0)).cos()
                    * 0.18;
            out[y * w + x] = v;
        }
    }
    out
}

fn make_stacked_series(series_n: usize, n: usize, seed: u64) -> Vec<TimeSeriesF32> {
    let n = n.max(2);
    let mut x = Vec::with_capacity(n);
    for i in 0..n {
        x.push(i as f32);
    }

    let mut out = Vec::with_capacity(series_n.max(1));
    for s in 0..series_n.max(1) {
        let mut y = Vec::with_capacity(n);
        let phase = s as f32 * seed_range(seed, 60, 0.4, 1.1);
        let f0 = seed_range(seed, 61 + s as u64, 0.006, 0.016);
        let f1 = seed_range(seed, 62 + s as u64, 0.08, 0.23);
        let base = seed_range(seed, 63, 0.8, 1.5);

        for i in 0..n {
            let t = i as f32;
            let v = (t * f0 + phase).sin() * 0.74 + (t * f1 + phase).sin() * 0.32 + base;
            y.push(v.max(0.0));
        }
        out.push(TimeSeriesF32::new(x.clone(), y).expect("sorted by construction"));
    }
    out
}

fn make_density_points(n: usize, seed: u64) -> Vec<Point> {
    let n = n.max(1);
    let mut out = Vec::with_capacity(n);

    let cx0 = seed_range(seed, 70, 0.32, 0.50);
    let cy0 = seed_range(seed, 71, 0.42, 0.62);
    let cx1 = seed_range(seed, 72, 0.52, 0.74);
    let cy1 = seed_range(seed, 73, 0.35, 0.58);

    for i in 0..n {
        let t = i as f32 * 0.0025;
        let (cx, cy) = if i % 2 == 0 { (cx0, cy0) } else { (cx1, cy1) };

        let dx = (t * seed_range(seed, 74, 2.3, 3.8)).sin() * 0.22
            + (t * seed_range(seed, 75, 0.12, 0.24)).cos() * 0.08;
        let dy = (t * seed_range(seed, 76, 2.1, 3.5)).cos() * 0.18
            + (t * seed_range(seed, 77, 0.10, 0.22)).sin() * 0.07;

        let x = cx + dx + (t * seed_range(seed, 78, 0.8, 1.2)).sin() * 0.03;
        let y = cy + dy + (t * seed_range(seed, 79, 0.9, 1.3)).cos() * 0.03;
        out.push(Point::new(x, y));
    }
    out
}

fn make_contour_values(w: usize, h: usize, seed: u64) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let cx = (w as f32) * seed_range(seed, 80, 0.42, 0.72);
    let cy = (h as f32) * seed_range(seed, 81, 0.30, 0.64);
    let spread = seed_range(seed, 82, 22.0, 34.0);
    let wx = seed_range(seed, 83, 10.0, 18.0);
    let wy = seed_range(seed, 84, 8.0, 14.0);

    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - cx) / (w as f32);
            let dy = (y as f32 - cy) / (h as f32);
            let r2 = dx * dx + dy * dy;
            let bump = (-r2 * spread).exp();
            let ripple = (dx * wx + seed_range(seed, 85, 0.0, 2.0)).sin()
                * (dy * wy + seed_range(seed, 86, 0.0, 2.0)).cos()
                * 0.25;
            out[y * w + x] = (bump + ripple) * 2.0 - 1.0;
        }
    }
    out
}

fn make_statistics_groups(groups_n: usize, points_per_group: usize, seed: u64) -> Vec<Vec<f32>> {
    let groups_n = groups_n.max(1);
    let points_per_group = points_per_group.max(8);

    let mut out = Vec::with_capacity(groups_n);
    for g in 0..groups_n {
        let mut vals = Vec::with_capacity(points_per_group);
        let shift = g as f32 * seed_range(seed, 90, 0.09, 0.20);
        let spread = 0.30 + (g as f32 * seed_range(seed, 91, 0.02, 0.06)).sin().abs() * 0.35;

        for i in 0..points_per_group {
            let t = i as f32 * 0.07;
            let v = (t + shift).sin() * spread
                + (t * seed_range(seed, 92, 0.15, 0.30) + shift).cos() * 0.18
                + shift * 0.6
                + seed_signed(seed, 3500 + (g as u64 * 400 + i as u64), 0.05);
            vals.push(v);
        }

        out.push(vals);
    }
    out
}

fn make_hierarchy_tree(seed: u64) -> HierarchyNode {
    let leaf = |name: &str, base: f32, stream: u64| {
        let v = (base + seed_signed(seed, stream, base * 0.28)).max(0.2);
        HierarchyNode::leaf(name, v)
    };

    HierarchyNode::node(
        "root",
        vec![
            HierarchyNode::node(
                "A",
                vec![
                    leaf("A-1", 6.0, 100),
                    leaf("A-2", 2.0, 101),
                    leaf("A-3", 4.0, 102),
                ],
            ),
            HierarchyNode::node(
                "B",
                vec![
                    leaf("B-1", 3.0, 110),
                    leaf("B-2", 7.0, 111),
                    leaf("B-3", 1.5, 112),
                    leaf("B-4", 2.2, 113),
                ],
            ),
            HierarchyNode::node(
                "C",
                vec![
                    leaf("C-1", 4.5, 120),
                    leaf("C-2", 1.2, 121),
                    leaf("C-3", 3.4, 122),
                ],
            ),
        ],
    )
}

fn make_graph_labels(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("N{i}")).collect()
}

fn make_graph_edges(n: usize, seed: u64) -> Vec<(usize, usize)> {
    let n = n.max(2);
    let mut out = Vec::new();

    let jump_a = seed_range(seed, 130, 5.0, 11.0).round() as usize;
    let jump_b = seed_range(seed, 131, 9.0, 17.0).round() as usize;

    for i in 0..n {
        out.push((i, (i + 1) % n));
    }

    for i in 0..n {
        if i % 3 == 0 {
            out.push((i, (i + jump_a.max(2)) % n));
        }
        if i % 5 == 0 {
            out.push((i, (i + jump_b.max(3)) % n));
        }
    }

    out
}

fn make_sankey_links(n: usize, seed: u64) -> Vec<(usize, usize, f32)> {
    let n = n.max(6);
    let cols = 3usize;
    let mut out = Vec::new();

    for i in 0..n {
        let col = i % cols;
        if col + 1 >= cols {
            continue;
        }

        let to_a = (i + 1).min(n - 1);
        let to_b = (i + cols).min(n - 1);

        let w1 = seed_range(seed, 200 + i as u64 * 2, 2.0, 9.0);
        let w2 = seed_range(seed, 201 + i as u64 * 2, 1.0, 6.5);

        if to_a != i {
            out.push((i, to_a, w1));
        }
        if to_b != i && to_b != to_a {
            out.push((i, to_b, w2));
        }
    }

    if out.is_empty() {
        out.push((0, 1, 3.0));
        out.push((1, 2, 2.0));
    }

    out
}

fn make_chord_matrix(n: usize, seed: u64) -> Vec<Vec<f32>> {
    let n = n.max(3);
    let mut m = vec![vec![0.0f32; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let t = ((i as f32 * 0.37) + (j as f32 * 0.23)) * seed_range(seed, 300, 0.8, 1.4)
                + seed_range(seed, 301, 0.0, 2.5);
            let w = (t.sin().abs() * seed_range(seed, 302, 2.0, 8.0)).max(0.0);
            m[i][j] = if (i + j) % 3 == 0 { w } else { w * 0.55 };
        }
    }

    m
}

fn make_network_model(mode: NetworkMode, seed: u64) -> anyhow::Result<NetworkChartModel> {
    match mode {
        NetworkMode::Graph => {
            let n = 48;
            NetworkChartModel::new_graph(make_graph_labels(n), make_graph_edges(n, seed))
        }
        NetworkMode::Sankey => {
            let n = 21;
            NetworkChartModel::new_sankey(make_graph_labels(n), make_sankey_links(n, seed))
        }
        NetworkMode::Chord => {
            let n = 14;
            NetworkChartModel::new_chord(make_graph_labels(n), make_chord_matrix(n, seed))
        }
    }
}

fn make_radar_dimensions() -> Vec<String> {
    vec![
        "Quality".into(),
        "Speed".into(),
        "Cost".into(),
        "Reliability".into(),
        "Scale".into(),
        "Latency".into(),
    ]
}

fn make_radar_series(seed: u64) -> Vec<Vec<f32>> {
    let mut series = Vec::new();
    for s in 0..3 {
        let mut row = Vec::new();
        for d in 0..6 {
            let base = 0.45 + (d as f32 * 0.08 + s as f32 * 0.13).sin() * 0.22;
            let v = (base + seed_signed(seed, 400 + (s as u64 * 10 + d as u64), 0.18))
                .clamp(0.05, 0.98);
            row.push(v);
        }
        series.push(row);
    }
    series
}

fn make_funnel_stages(seed: u64) -> Vec<(String, f32)> {
    let mut current = seed_range(seed, 500, 10_000.0, 18_000.0);
    let mut out = Vec::new();

    let names = ["Visits", "Signups", "Trials", "Paid", "Renew"];
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            let retention = seed_range(seed, 501 + i as u64, 0.38, 0.76);
            current *= retention;
        }
        out.push(((*name).to_string(), current.max(100.0)));
    }

    out
}

fn make_geo_shapes(seed: u64) -> Vec<Vec<Point>> {
    let mut shapes = Vec::new();

    let mut coast = Vec::new();
    let f0 = seed_range(seed, 600, 1.3, 2.0);
    let f1 = seed_range(seed, 601, 3.4, 5.2);

    for i in 0..220 {
        let t = i as f32 / 219.0;
        let x = t * 10.0;
        let y = (t * std::f32::consts::TAU * f0).sin() * 0.9
            + (t * std::f32::consts::TAU * f1).sin() * 0.25
            + seed_signed(seed, 602 + i as u64, 0.04);
        coast.push(Point::new(x, y));
    }
    shapes.push(coast);

    let mut island = Vec::new();
    let cx = seed_range(seed, 610, 5.8, 7.4);
    let cy = seed_range(seed, 611, -2.1, -1.0);
    let rx = seed_range(seed, 612, 0.8, 1.6);
    let ry = seed_range(seed, 613, 0.5, 0.9);

    for i in 0..=64 {
        let a = i as f32 / 64.0 * std::f32::consts::TAU;
        island.push(Point::new(
            cx + a.cos() * rx,
            cy + a.sin() * ry + (a * seed_range(seed, 614, 2.0, 4.0)).sin() * 0.08,
        ));
    }
    shapes.push(island);

    shapes
}

fn next_stacked_mode(mode: StackedAreaMode) -> StackedAreaMode {
    match mode {
        StackedAreaMode::Stacked => StackedAreaMode::Streamgraph,
        StackedAreaMode::Streamgraph => StackedAreaMode::Stacked,
    }
}

fn stacked_mode_label(mode: StackedAreaMode) -> &'static str {
    match mode {
        StackedAreaMode::Stacked => "Stacked",
        StackedAreaMode::Streamgraph => "Streamgraph",
    }
}

fn next_hierarchy_mode(mode: HierarchyMode) -> HierarchyMode {
    match mode {
        HierarchyMode::Treemap => HierarchyMode::Icicle,
        HierarchyMode::Icicle => HierarchyMode::Sunburst,
        HierarchyMode::Sunburst => HierarchyMode::Packing,
        HierarchyMode::Packing => HierarchyMode::Treemap,
    }
}

fn hierarchy_mode_label(mode: HierarchyMode) -> &'static str {
    match mode {
        HierarchyMode::Treemap => "Treemap",
        HierarchyMode::Icicle => "Icicle",
        HierarchyMode::Sunburst => "Sunburst",
        HierarchyMode::Packing => "Packing",
    }
}

fn next_network_mode(mode: NetworkMode) -> NetworkMode {
    match mode {
        NetworkMode::Graph => NetworkMode::Sankey,
        NetworkMode::Sankey => NetworkMode::Chord,
        NetworkMode::Chord => NetworkMode::Graph,
    }
}

fn network_mode_label(mode: NetworkMode) -> &'static str {
    match mode {
        NetworkMode::Graph => "Graph",
        NetworkMode::Sankey => "Sankey",
        NetworkMode::Chord => "Chord",
    }
}

fn next_polar_mode(mode: PolarChartMode) -> PolarChartMode {
    match mode {
        PolarChartMode::Radar => PolarChartMode::Polar,
        PolarChartMode::Polar => PolarChartMode::Parallel,
        PolarChartMode::Parallel => PolarChartMode::Radar,
    }
}

fn polar_mode_label(mode: PolarChartMode) -> &'static str {
    match mode {
        PolarChartMode::Radar => "Radar",
        PolarChartMode::Polar => "Polar",
        PolarChartMode::Parallel => "Parallel",
    }
}
