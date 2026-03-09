//! Theme trait and core types

use crate::tokens::*;
use std::sync::Arc;

/// Color scheme variant
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}

impl ColorScheme {
    /// Toggle to the opposite scheme
    pub fn toggle(self) -> Self {
        match self {
            ColorScheme::Light => ColorScheme::Dark,
            ColorScheme::Dark => ColorScheme::Light,
        }
    }
}

/// The main theme trait that all themes must implement
pub trait Theme: Send + Sync + std::fmt::Debug {
    /// Get the theme name for debugging
    fn name(&self) -> &str;

    /// Get the current color scheme
    fn color_scheme(&self) -> ColorScheme;

    /// Get color tokens
    fn colors(&self) -> &ColorTokens;

    /// Get typography tokens
    fn typography(&self) -> &TypographyTokens;

    /// Get spacing tokens
    fn spacing(&self) -> &SpacingTokens;

    /// Get radius tokens
    fn radii(&self) -> &RadiusTokens;

    /// Get shadow tokens
    fn shadows(&self) -> &ShadowTokens;

    /// Get animation tokens
    fn animations(&self) -> &AnimationTokens;

    /// Get semantic component tokens derived from primitive scales.
    fn components(&self) -> ComponentTokens {
        ComponentTokens::from_primitives(
            self.spacing(),
            self.radii(),
            self.typography(),
            self.shadows(),
        )
    }
}

/// A theme bundle containing both light and dark variants
#[derive(Clone)]
pub struct ThemeBundle {
    /// Theme name
    pub name: String,
    /// Light theme variant
    pub light: Arc<dyn Theme>,
    /// Dark theme variant
    pub dark: Arc<dyn Theme>,
}

impl ThemeBundle {
    /// Create a new theme bundle
    pub fn new(
        name: impl Into<String>,
        light: impl Theme + 'static,
        dark: impl Theme + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            light: Arc::new(light),
            dark: Arc::new(dark),
        }
    }

    /// Get the theme for the specified color scheme
    pub fn for_scheme(&self, scheme: ColorScheme) -> Arc<dyn Theme> {
        match scheme {
            ColorScheme::Light => Arc::clone(&self.light),
            ColorScheme::Dark => Arc::clone(&self.dark),
        }
    }
}

impl std::fmt::Debug for ThemeBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeBundle")
            .field("name", &self.name)
            .finish()
    }
}
