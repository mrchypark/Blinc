//! Global theme state singleton
//!
//! ThemeState is designed to avoid triggering full layout rebuilds on theme changes.
//! - Visual tokens (colors, shadows) can be animated and only trigger repaints
//! - Layout tokens (spacing, typography, radii) trigger partial layout recomputation

use crate::theme::{ColorScheme, ThemeBundle};
use crate::tokens::*;
use blinc_animation::{AnimatedValue, AnimationScheduler, SchedulerHandle, SpringConfig};
use blinc_core::Color;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex, OnceLock, RwLock};

/// Global theme state instance
static THEME_STATE: OnceLock<ThemeState> = OnceLock::new();

/// Global redraw callback - set by the app layer to trigger UI updates
static REDRAW_CALLBACK: Mutex<Option<fn()>> = Mutex::new(None);

/// Set the redraw callback function
///
/// This should be called by the app layer (e.g., blinc_app) to register
/// a function that triggers UI redraws when theme changes.
pub fn set_redraw_callback(callback: fn()) {
    *REDRAW_CALLBACK.lock().unwrap() = Some(callback);
}

/// Trigger a redraw via the registered callback
fn trigger_redraw() {
    if let Some(callback) = *REDRAW_CALLBACK.lock().unwrap() {
        callback();
    }
}

/// Theme transition animation state
#[derive(Default)]
struct ThemeTransition {
    /// Animated progress value (0.0 = old theme, 1.0 = new theme)
    /// Uses AnimatedValue which is automatically ticked by the animation scheduler
    progress: Option<AnimatedValue>,
    /// Colors from the old theme (for interpolation)
    from_colors: Option<ColorTokens>,
    /// Colors from the new theme (target)
    to_colors: Option<ColorTokens>,
}

/// Global theme state - accessed directly by widgets during render
pub struct ThemeState {
    /// The current theme bundle (light/dark pair)
    bundle: ThemeBundle,

    /// Current color scheme
    scheme: RwLock<ColorScheme>,

    /// Current color tokens (can be animated)
    colors: RwLock<ColorTokens>,

    /// Current shadow tokens (can be animated)
    shadows: RwLock<ShadowTokens>,

    /// Current opacity tokens
    opacities: RwLock<OpacityTokens>,

    /// Current spacing tokens
    spacing: RwLock<SpacingTokens>,

    /// Current typography tokens
    typography: RwLock<TypographyTokens>,

    /// Current radius tokens
    radii: RwLock<RadiusTokens>,

    /// Current semantic component tokens
    components: RwLock<ComponentTokens>,

    /// Current animation tokens
    animations: RwLock<AnimationTokens>,

    /// Dynamic color overrides
    color_overrides: RwLock<FxHashMap<ColorToken, Color>>,

    /// Dynamic spacing overrides
    spacing_overrides: RwLock<FxHashMap<SpacingToken, f32>>,

    /// Dynamic opacity overrides
    opacity_overrides: RwLock<FxHashMap<OpacityToken, f32>>,

    /// Dynamic radius overrides
    radius_overrides: RwLock<FxHashMap<RadiusToken, f32>>,

    /// Flag indicating theme needs repaint (colors changed)
    needs_repaint: AtomicBool,

    /// Flag indicating theme needs layout (spacing/typography changed)
    needs_layout: AtomicBool,

    /// Animation scheduler handle (set after window creation)
    scheduler_handle: RwLock<Option<SchedulerHandle>>,

    /// Theme transition animation state
    transition: Mutex<ThemeTransition>,
}

impl ThemeState {
    fn effective_spacing_tokens(&self) -> SpacingTokens {
        let mut spacing = self.spacing.read().unwrap().clone();
        for (token, value) in self.spacing_overrides.read().unwrap().iter() {
            match token {
                SpacingToken::Space0 => spacing.space_0 = *value,
                SpacingToken::Space0_5 => spacing.space_0_5 = *value,
                SpacingToken::Space1 => spacing.space_1 = *value,
                SpacingToken::Space1_5 => spacing.space_1_5 = *value,
                SpacingToken::Space2 => spacing.space_2 = *value,
                SpacingToken::Space2_5 => spacing.space_2_5 = *value,
                SpacingToken::Space3 => spacing.space_3 = *value,
                SpacingToken::Space3_5 => spacing.space_3_5 = *value,
                SpacingToken::Space4 => spacing.space_4 = *value,
                SpacingToken::Space5 => spacing.space_5 = *value,
                SpacingToken::Space6 => spacing.space_6 = *value,
                SpacingToken::Space7 => spacing.space_7 = *value,
                SpacingToken::Space8 => spacing.space_8 = *value,
                SpacingToken::Space9 => spacing.space_9 = *value,
                SpacingToken::Space10 => spacing.space_10 = *value,
                SpacingToken::Space11 => spacing.space_11 = *value,
                SpacingToken::Space12 => spacing.space_12 = *value,
                SpacingToken::Space14 => spacing.space_14 = *value,
                SpacingToken::Space16 => spacing.space_16 = *value,
                SpacingToken::Space20 => spacing.space_20 = *value,
                SpacingToken::Space24 => spacing.space_24 = *value,
                SpacingToken::Space28 => spacing.space_28 = *value,
                SpacingToken::Space32 => spacing.space_32 = *value,
            }
        }
        spacing
    }

    fn effective_radius_tokens(&self) -> RadiusTokens {
        let mut radii = self.radii.read().unwrap().clone();
        for (token, value) in self.radius_overrides.read().unwrap().iter() {
            match token {
                RadiusToken::None => radii.radius_none = *value,
                RadiusToken::Sm => radii.radius_sm = *value,
                RadiusToken::Default => radii.radius_default = *value,
                RadiusToken::Md => radii.radius_md = *value,
                RadiusToken::Lg => radii.radius_lg = *value,
                RadiusToken::Xl => radii.radius_xl = *value,
                RadiusToken::Xxl => radii.radius_2xl = *value,
                RadiusToken::Xxxl => radii.radius_3xl = *value,
                RadiusToken::Full => radii.radius_full = *value,
            }
        }
        radii
    }

    fn recompute_component_tokens(&self) {
        let spacing = self.effective_spacing_tokens();
        let radii = self.effective_radius_tokens();
        let typography = self.typography.read().unwrap().clone();
        let shadows = self.shadows.read().unwrap().clone();

        *self.components.write().unwrap() =
            ComponentTokens::from_primitives(&spacing, &radii, &typography, &shadows);
    }

    /// Initialize the global theme state (call once at app startup)
    pub fn init(bundle: ThemeBundle, scheme: ColorScheme) {
        let theme = bundle.for_scheme(scheme);

        let state = ThemeState {
            bundle,
            scheme: RwLock::new(scheme),
            colors: RwLock::new(theme.colors().clone()),
            shadows: RwLock::new(theme.shadows().clone()),
            opacities: RwLock::new(OpacityTokens::default()),
            spacing: RwLock::new(theme.spacing().clone()),
            typography: RwLock::new(theme.typography().clone()),
            radii: RwLock::new(theme.radii().clone()),
            components: RwLock::new(theme.components()),
            animations: RwLock::new(theme.animations().clone()),
            color_overrides: RwLock::new(FxHashMap::default()),
            spacing_overrides: RwLock::new(FxHashMap::default()),
            opacity_overrides: RwLock::new(FxHashMap::default()),
            radius_overrides: RwLock::new(FxHashMap::default()),
            needs_repaint: AtomicBool::new(false),
            needs_layout: AtomicBool::new(false),
            scheduler_handle: RwLock::new(None),
            transition: Mutex::new(ThemeTransition::default()),
        };

        let _ = THEME_STATE.set(state);
    }

    /// Set the animation scheduler for theme transitions
    ///
    /// This should be called by the app layer after the window is created
    /// to enable animated theme transitions.
    pub fn set_scheduler(&self, scheduler: &Arc<Mutex<AnimationScheduler>>) {
        let handle = scheduler.lock().unwrap().handle();
        *self.scheduler_handle.write().unwrap() = Some(handle);
    }

    /// Initialize with platform-native theme and system color scheme
    ///
    /// Detects the current OS and uses the appropriate native theme:
    /// - macOS: Apple Human Interface Guidelines theme
    /// - Windows: Fluent Design System 2 theme
    /// - Linux: GNOME Adwaita theme
    pub fn init_default() {
        use crate::platform::detect_system_color_scheme;
        use crate::themes::platform::platform_theme_bundle;

        let bundle = platform_theme_bundle();
        let scheme = detect_system_color_scheme();
        Self::init(bundle, scheme);
    }

    /// Get the global theme state instance
    pub fn get() -> &'static ThemeState {
        THEME_STATE
            .get()
            .expect("ThemeState not initialized. Call ThemeState::init() at app startup.")
    }

    /// Try to get the global theme state (returns None if not initialized)
    pub fn try_get() -> Option<&'static ThemeState> {
        THEME_STATE.get()
    }

    // ========== Color Scheme ==========

    /// Get the current color scheme
    pub fn scheme(&self) -> ColorScheme {
        *self.scheme.read().unwrap()
    }

    /// Set the color scheme (animates colors if scheduler is available)
    pub fn set_scheme(&self, scheme: ColorScheme) {
        let mut current = self.scheme.write().unwrap();
        if *current != scheme {
            tracing::debug!(
                "ThemeState::set_scheme - switching from {:?} to {:?}",
                *current,
                scheme
            );
            // Get current colors before switching
            let old_colors = self.colors.read().unwrap().clone();

            *current = scheme;
            drop(current);

            // Get new theme tokens
            let theme = self.bundle.for_scheme(scheme);
            let new_colors = theme.colors().clone();

            // Update non-color tokens immediately (they don't animate)
            *self.shadows.write().unwrap() = theme.shadows().clone();
            *self.spacing.write().unwrap() = theme.spacing().clone();
            *self.typography.write().unwrap() = theme.typography().clone();
            *self.radii.write().unwrap() = theme.radii().clone();
            *self.components.write().unwrap() = theme.components();
            *self.animations.write().unwrap() = theme.animations().clone();

            // Try to animate colors if scheduler handle is available
            let handle_opt = self.scheduler_handle.read().unwrap().clone();
            if let Some(handle) = handle_opt {
                // Start animated transition using AnimatedValue
                let mut transition = self.transition.lock().unwrap();
                transition.from_colors = Some(old_colors.clone());
                transition.to_colors = Some(new_colors.clone());

                // Create AnimatedValue for progress (0 to 100, scaled to avoid spring epsilon issues)
                // The animation scheduler's background thread will tick this automatically
                let mut progress = AnimatedValue::new(handle, 0.0, SpringConfig::gentle());
                progress.set_target(100.0);
                transition.progress = Some(progress);

                // Initialize colors to starting point (old colors at progress=0)
                // This ensures immediate visual feedback before first tick
                drop(transition);
                *self.colors.write().unwrap() = old_colors;
            } else {
                // No scheduler, instant swap
                *self.colors.write().unwrap() = new_colors;
            }

            // Mark for repaint and layout
            self.needs_repaint.store(true, Ordering::SeqCst);
            self.needs_layout.store(true, Ordering::SeqCst);

            // Trigger UI redraw
            trigger_redraw();
        }
    }

    /// Update theme colors based on animation progress
    ///
    /// This should be called during the render loop to update interpolated colors.
    /// Returns true if animation is still in progress and needs more frames.
    pub fn tick(&self) -> bool {
        let mut transition = self.transition.lock().unwrap();

        // Check if we have an active animation
        let progress_opt = transition.progress.as_ref();
        if progress_opt.is_none() {
            return false;
        }

        let progress_anim = transition.progress.as_ref().unwrap();

        // Get current animated value (0-100 range, normalize to 0-1)
        let raw_progress = progress_anim.get();
        let progress = (raw_progress / 100.0).clamp(0.0, 1.0);

        // Check if animation has reached target (within threshold)
        // AnimatedValue.is_animating() just checks spring existence, not actual progress
        let at_target = (raw_progress - 100.0).abs() < 1.0;

        tracing::trace!(
            "Theme tick: raw={:.1}, progress={:.3}, at_target={}",
            raw_progress,
            progress,
            at_target
        );

        // Interpolate colors based on progress
        if let (Some(ref from), Some(ref to)) = (&transition.from_colors, &transition.to_colors) {
            let interpolated = interpolate_color_tokens(from, to, progress);
            drop(transition);
            *self.colors.write().unwrap() = interpolated;

            if at_target {
                // Animation complete - clean up
                let mut transition = self.transition.lock().unwrap();
                transition.progress = None;
                transition.from_colors = None;
                transition.to_colors = None;
                return false;
            }

            // Animation still in progress - trigger rebuild so colors are re-read
            trigger_redraw();
            return true;
        }

        // No colors to interpolate, end animation
        transition.progress = None;
        false
    }

    /// Check if a theme transition animation is in progress
    pub fn is_animating(&self) -> bool {
        let transition = self.transition.lock().unwrap();
        transition
            .progress
            .as_ref()
            .map(|p| p.is_animating())
            .unwrap_or(false)
    }

    /// Toggle between light and dark mode
    pub fn toggle_scheme(&self) {
        let current = self.scheme();
        self.set_scheme(current.toggle());
    }

    // ========== Color Access ==========

    /// Get a color token value (checks override first)
    pub fn color(&self, token: ColorToken) -> Color {
        // Check override first
        if let Some(color) = self.color_overrides.read().unwrap().get(&token) {
            return *color;
        }
        self.colors.read().unwrap().get(token)
    }

    /// Get all color tokens
    pub fn colors(&self) -> ColorTokens {
        self.colors.read().unwrap().clone()
    }

    /// Set a color override (triggers repaint only)
    pub fn set_color_override(&self, token: ColorToken, color: Color) {
        self.color_overrides.write().unwrap().insert(token, color);
        self.needs_repaint.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    /// Remove a color override
    pub fn remove_color_override(&self, token: ColorToken) {
        self.color_overrides.write().unwrap().remove(&token);
        self.needs_repaint.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    // ========== CSS Variable Generation ==========

    /// Generate a CSS variable map from all color tokens.
    ///
    /// Returns a `HashMap<String, String>` where keys are variable names
    /// (without `--` prefix) and values are hex color strings.
    /// Variable names match the `theme()` CSS function token names.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let vars = ThemeState::get().to_css_variable_map();
    /// // vars["text-primary"] == "#1a1a2e"
    /// // vars["surface"] == "#ffffff"
    /// ```
    pub fn to_css_variable_map(&self) -> HashMap<String, String> {
        fn hex(c: Color) -> String {
            if c.a < 1.0 {
                format!(
                    "rgba({},{},{},{})",
                    (c.r * 255.0) as u8,
                    (c.g * 255.0) as u8,
                    (c.b * 255.0) as u8,
                    c.a
                )
            } else {
                format!(
                    "#{:02x}{:02x}{:02x}",
                    (c.r * 255.0) as u8,
                    (c.g * 255.0) as u8,
                    (c.b * 255.0) as u8
                )
            }
        }

        let mut vars = HashMap::with_capacity(70);

        // Use self.color() which checks overrides first
        vars.insert("primary".into(), hex(self.color(ColorToken::Primary)));
        vars.insert(
            "primary-hover".into(),
            hex(self.color(ColorToken::PrimaryHover)),
        );
        vars.insert(
            "primary-active".into(),
            hex(self.color(ColorToken::PrimaryActive)),
        );
        vars.insert("secondary".into(), hex(self.color(ColorToken::Secondary)));
        vars.insert(
            "secondary-hover".into(),
            hex(self.color(ColorToken::SecondaryHover)),
        );
        vars.insert(
            "secondary-active".into(),
            hex(self.color(ColorToken::SecondaryActive)),
        );
        vars.insert("success".into(), hex(self.color(ColorToken::Success)));
        vars.insert("success-bg".into(), hex(self.color(ColorToken::SuccessBg)));
        vars.insert("warning".into(), hex(self.color(ColorToken::Warning)));
        vars.insert("warning-bg".into(), hex(self.color(ColorToken::WarningBg)));
        vars.insert("error".into(), hex(self.color(ColorToken::Error)));
        vars.insert("error-bg".into(), hex(self.color(ColorToken::ErrorBg)));
        vars.insert("info".into(), hex(self.color(ColorToken::Info)));
        vars.insert("info-bg".into(), hex(self.color(ColorToken::InfoBg)));
        vars.insert("background".into(), hex(self.color(ColorToken::Background)));
        vars.insert("surface".into(), hex(self.color(ColorToken::Surface)));
        vars.insert(
            "surface-elevated".into(),
            hex(self.color(ColorToken::SurfaceElevated)),
        );
        vars.insert(
            "surface-overlay".into(),
            hex(self.color(ColorToken::SurfaceOverlay)),
        );
        vars.insert(
            "text-primary".into(),
            hex(self.color(ColorToken::TextPrimary)),
        );
        vars.insert(
            "text-secondary".into(),
            hex(self.color(ColorToken::TextSecondary)),
        );
        vars.insert(
            "text-tertiary".into(),
            hex(self.color(ColorToken::TextTertiary)),
        );
        vars.insert(
            "text-inverse".into(),
            hex(self.color(ColorToken::TextInverse)),
        );
        vars.insert("text-link".into(), hex(self.color(ColorToken::TextLink)));
        vars.insert("border".into(), hex(self.color(ColorToken::Border)));
        vars.insert(
            "border-secondary".into(),
            hex(self.color(ColorToken::BorderSecondary)),
        );
        vars.insert(
            "border-hover".into(),
            hex(self.color(ColorToken::BorderHover)),
        );
        vars.insert(
            "border-focus".into(),
            hex(self.color(ColorToken::BorderFocus)),
        );
        vars.insert(
            "border-error".into(),
            hex(self.color(ColorToken::BorderError)),
        );
        vars.insert("input-bg".into(), hex(self.color(ColorToken::InputBg)));
        vars.insert(
            "input-bg-hover".into(),
            hex(self.color(ColorToken::InputBgHover)),
        );
        vars.insert(
            "input-bg-focus".into(),
            hex(self.color(ColorToken::InputBgFocus)),
        );
        vars.insert(
            "input-bg-disabled".into(),
            hex(self.color(ColorToken::InputBgDisabled)),
        );
        vars.insert("selection".into(), hex(self.color(ColorToken::Selection)));
        vars.insert(
            "selection-text".into(),
            hex(self.color(ColorToken::SelectionText)),
        );
        vars.insert("accent".into(), hex(self.color(ColorToken::Accent)));
        vars.insert(
            "accent-subtle".into(),
            hex(self.color(ColorToken::AccentSubtle)),
        );
        vars.insert(
            "tooltip-bg".into(),
            hex(self.color(ColorToken::TooltipBackground)),
        );
        vars.insert(
            "tooltip-text".into(),
            hex(self.color(ColorToken::TooltipText)),
        );

        let components = self.components();
        vars.insert("control-height-sm".into(), px(components.control.height_sm));
        vars.insert("control-height-md".into(), px(components.control.height_md));
        vars.insert("control-height-lg".into(), px(components.control.height_lg));
        vars.insert("control-px-sm".into(), px(components.control.padding_x_sm));
        vars.insert("control-px-md".into(), px(components.control.padding_x_md));
        vars.insert("control-px-lg".into(), px(components.control.padding_x_lg));
        vars.insert("control-py-sm".into(), px(components.control.padding_y_sm));
        vars.insert("control-py-md".into(), px(components.control.padding_y_md));
        vars.insert("control-py-lg".into(), px(components.control.padding_y_lg));
        vars.insert("control-radius-sm".into(), px(components.control.radius_sm));
        vars.insert("control-radius-md".into(), px(components.control.radius_md));
        vars.insert("control-radius-lg".into(), px(components.control.radius_lg));
        vars.insert(
            "control-corner-shape".into(),
            components.control.corner_shape_rest.to_string(),
        );
        vars.insert(
            "control-corner-shape-hover".into(),
            components.control.corner_shape_hover.to_string(),
        );
        vars.insert("container-radius".into(), px(components.container.radius));
        vars.insert("container-padding".into(), px(components.container.padding));
        vars.insert(
            "container-padding-compact".into(),
            px(components.container.padding_compact),
        );
        vars.insert(
            "container-header-gap".into(),
            px(components.container.header_gap),
        );
        vars.insert(
            "container-footer-gap".into(),
            px(components.container.footer_gap),
        );
        vars.insert(
            "container-section-gap".into(),
            px(components.container.section_gap),
        );
        vars.insert(
            "container-corner-shape".into(),
            components.container.corner_shape.to_string(),
        );
        vars.insert("overlay-radius".into(), px(components.overlay.radius));
        vars.insert("overlay-px".into(), px(components.overlay.padding_x));
        vars.insert("overlay-py".into(), px(components.overlay.padding_y));
        vars.insert(
            "overlay-item-px".into(),
            px(components.overlay.item_padding_x),
        );
        vars.insert(
            "overlay-item-py".into(),
            px(components.overlay.item_padding_y),
        );
        vars.insert("overlay-gap".into(), px(components.overlay.gap));
        vars.insert(
            "overlay-shadow".into(),
            css_shadow(&components.overlay.shadow),
        );
        vars.insert(
            "overlay-corner-shape".into(),
            components.overlay.corner_shape.to_string(),
        );
        vars.insert("type-action-sm".into(), px(components.typography.action_sm));
        vars.insert("type-action-md".into(), px(components.typography.action_md));
        vars.insert("type-action-lg".into(), px(components.typography.action_lg));
        vars.insert("type-body-sm".into(), px(components.typography.body_sm));
        vars.insert("type-body-md".into(), px(components.typography.body_md));
        vars.insert("type-body-lg".into(), px(components.typography.body_lg));
        vars.insert("type-label-sm".into(), px(components.typography.label_sm));
        vars.insert("type-label-md".into(), px(components.typography.label_md));
        vars.insert("type-label-lg".into(), px(components.typography.label_lg));
        vars.insert("type-helper".into(), px(components.typography.helper));
        vars.insert("type-badge".into(), px(components.typography.badge));
        vars.insert("type-title".into(), px(components.typography.title));
        vars.insert(
            "compact-badge-radius".into(),
            px(components.compact.badge_radius),
        );
        vars.insert(
            "compact-badge-px".into(),
            px(components.compact.badge_padding_x),
        );
        vars.insert(
            "compact-badge-py".into(),
            px(components.compact.badge_padding_y),
        );
        vars.insert(
            "compact-kbd-radius".into(),
            px(components.compact.kbd_radius),
        );
        vars.insert(
            "compact-kbd-px".into(),
            px(components.compact.kbd_padding_x),
        );
        vars.insert(
            "compact-kbd-py".into(),
            px(components.compact.kbd_padding_y),
        );
        vars.insert(
            "compact-cluster-gap-sm".into(),
            px(components.compact.cluster_gap_sm),
        );
        vars.insert(
            "compact-cluster-gap-md".into(),
            px(components.compact.cluster_gap_md),
        );
        vars.insert(
            "compact-progress-height-sm".into(),
            px(components.compact.progress_height_sm),
        );
        vars.insert(
            "compact-progress-height-md".into(),
            px(components.compact.progress_height_md),
        );
        vars.insert(
            "compact-progress-height-lg".into(),
            px(components.compact.progress_height_lg),
        );
        vars.insert(
            "compact-switch-inset".into(),
            px(components.compact.switch_inset),
        );

        vars
    }

    /// Get semantic component tokens
    pub fn components(&self) -> ComponentTokens {
        self.components.read().unwrap().clone()
    }

    // ========== Opacity Access ==========

    /// Get an opacity token value (checks override first)
    pub fn opacity_value(&self, token: OpacityToken) -> f32 {
        if let Some(value) = self.opacity_overrides.read().unwrap().get(&token) {
            return *value;
        }
        self.opacities.read().unwrap().get(token)
    }

    /// Get all opacity tokens
    pub fn opacities(&self) -> OpacityTokens {
        self.opacities.read().unwrap().clone()
    }

    /// Set an opacity override (triggers repaint only)
    pub fn set_opacity_override(&self, token: OpacityToken, value: f32) {
        self.opacity_overrides.write().unwrap().insert(token, value);
        self.needs_repaint.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    /// Remove an opacity override
    pub fn remove_opacity_override(&self, token: OpacityToken) {
        self.opacity_overrides.write().unwrap().remove(&token);
        self.needs_repaint.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    // ========== Spacing Access ==========

    /// Get a spacing token value (checks override first)
    pub fn spacing_value(&self, token: SpacingToken) -> f32 {
        if let Some(value) = self.spacing_overrides.read().unwrap().get(&token) {
            return *value;
        }
        self.spacing.read().unwrap().get(token)
    }

    /// Get all spacing tokens
    pub fn spacing(&self) -> SpacingTokens {
        self.spacing.read().unwrap().clone()
    }

    /// Set a spacing override (triggers layout)
    pub fn set_spacing_override(&self, token: SpacingToken, value: f32) {
        self.spacing_overrides.write().unwrap().insert(token, value);
        self.recompute_component_tokens();
        self.needs_layout.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    /// Remove a spacing override
    pub fn remove_spacing_override(&self, token: SpacingToken) {
        self.spacing_overrides.write().unwrap().remove(&token);
        self.recompute_component_tokens();
        self.needs_layout.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    // ========== Typography Access ==========

    /// Get all typography tokens
    pub fn typography(&self) -> TypographyTokens {
        self.typography.read().unwrap().clone()
    }

    // ========== Radius Access ==========

    /// Get a radius token value (checks override first)
    pub fn radius(&self, token: RadiusToken) -> f32 {
        if let Some(value) = self.radius_overrides.read().unwrap().get(&token) {
            return *value;
        }
        self.radii.read().unwrap().get(token)
    }

    /// Get all radius tokens
    pub fn radii(&self) -> RadiusTokens {
        self.radii.read().unwrap().clone()
    }

    /// Set a radius override (triggers repaint - radii don't affect layout)
    pub fn set_radius_override(&self, token: RadiusToken, value: f32) {
        self.radius_overrides.write().unwrap().insert(token, value);
        self.recompute_component_tokens();
        self.needs_repaint.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    /// Remove a radius override
    pub fn remove_radius_override(&self, token: RadiusToken) {
        self.radius_overrides.write().unwrap().remove(&token);
        self.recompute_component_tokens();
        self.needs_repaint.store(true, Ordering::SeqCst);
        trigger_redraw();
    }

    // ========== Shadow Access ==========

    /// Get all shadow tokens
    pub fn shadows(&self) -> ShadowTokens {
        self.shadows.read().unwrap().clone()
    }

    // ========== Animation Access ==========

    /// Get all animation tokens
    pub fn animations(&self) -> AnimationTokens {
        self.animations.read().unwrap().clone()
    }

    // ========== Dirty Flags ==========

    /// Check if theme changes require repaint
    pub fn needs_repaint(&self) -> bool {
        self.needs_repaint.load(Ordering::SeqCst)
    }

    /// Clear the repaint flag
    pub fn clear_repaint(&self) {
        self.needs_repaint.store(false, Ordering::SeqCst);
    }

    /// Check if theme changes require layout
    pub fn needs_layout(&self) -> bool {
        self.needs_layout.load(Ordering::SeqCst)
    }

    /// Clear the layout flag
    pub fn clear_layout(&self) {
        self.needs_layout.store(false, Ordering::SeqCst);
    }

    // ========== Override Management ==========

    /// Clear all overrides
    pub fn clear_overrides(&self) {
        self.color_overrides.write().unwrap().clear();
        self.spacing_overrides.write().unwrap().clear();
        self.opacity_overrides.write().unwrap().clear();
        self.radius_overrides.write().unwrap().clear();
        self.recompute_component_tokens();
        self.needs_repaint.store(true, Ordering::SeqCst);
        self.needs_layout.store(true, Ordering::SeqCst);
        trigger_redraw();
    }
}

/// Interpolate between two color token sets
fn interpolate_color_tokens(from: &ColorTokens, to: &ColorTokens, t: f32) -> ColorTokens {
    ColorTokens::lerp(from, to, t)
}

fn px(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}px")
    } else {
        format!("{value:.2}px")
    }
}

fn css_shadow(shadow: &Shadow) -> String {
    if shadow.color.a == 0.0 {
        return "none".into();
    }

    format!(
        "{} {} {} {} rgba({},{},{},{})",
        px(shadow.offset_x),
        px(shadow.offset_y),
        px(shadow.blur),
        px(shadow.spread),
        (shadow.color.r * 255.0) as u8,
        (shadow.color.g * 255.0) as u8,
        (shadow.color.b * 255.0) as u8,
        shadow.color.a
    )
}
