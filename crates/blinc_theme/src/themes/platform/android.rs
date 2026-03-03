//! Android Material You theme

use crate::theme::{ColorScheme, Theme, ThemeBundle};
use crate::themes::BlincTheme;
use crate::tokens::*;

/// Android-native theme following Material You design
#[derive(Clone, Debug)]
pub struct AndroidTheme {
    inner: BlincTheme,
}

impl AndroidTheme {
    pub fn light() -> Self {
        // TODO: Customize with Material You colors
        Self {
            inner: BlincTheme::light(),
        }
    }

    pub fn dark() -> Self {
        // TODO: Customize with Material You colors
        Self {
            inner: BlincTheme::dark(),
        }
    }

    pub fn bundle() -> ThemeBundle {
        ThemeBundle::new("Android", Self::light(), Self::dark())
    }
}

impl Theme for AndroidTheme {
    fn name(&self) -> &str {
        "Android"
    }

    fn color_scheme(&self) -> ColorScheme {
        self.inner.color_scheme()
    }

    fn colors(&self) -> &ColorTokens {
        self.inner.colors()
    }

    fn typography(&self) -> &TypographyTokens {
        self.inner.typography()
    }

    fn spacing(&self) -> &SpacingTokens {
        self.inner.spacing()
    }

    fn opacities(&self) -> &OpacityTokens {
        self.inner.opacities()
    }

    fn radii(&self) -> &RadiusTokens {
        self.inner.radii()
    }

    fn shadows(&self) -> &ShadowTokens {
        self.inner.shadows()
    }

    fn animations(&self) -> &AnimationTokens {
        self.inner.animations()
    }
}
