//! Shadow tokens for theming

use blinc_core::Color;

/// Semantic shadow token keys for dynamic access
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ShadowToken {
    Sm,
    Default,
    Md,
    Lg,
    Xl,
    Xxl,
    Inner,
    None,
}

/// A box shadow definition
#[derive(Clone, Debug)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Shadow {
    pub const fn new(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            spread,
            color,
        }
    }

    pub const fn none() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: Color::TRANSPARENT,
        }
    }

    /// Linear interpolation between two shadows
    pub fn lerp(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            offset_x: from.offset_x + (to.offset_x - from.offset_x) * t,
            offset_y: from.offset_y + (to.offset_y - from.offset_y) * t,
            blur: from.blur + (to.blur - from.blur) * t,
            spread: from.spread + (to.spread - from.spread) * t,
            color: Color::lerp(&from.color, &to.color, t),
        }
    }
}

impl Default for Shadow {
    fn default() -> Self {
        Self::none()
    }
}

impl From<Shadow> for blinc_core::Shadow {
    fn from(shadow: Shadow) -> Self {
        blinc_core::Shadow {
            offset_x: shadow.offset_x,
            offset_y: shadow.offset_y,
            blur: shadow.blur,
            spread: shadow.spread,
            color: shadow.color,
        }
    }
}

impl From<&Shadow> for blinc_core::Shadow {
    fn from(shadow: &Shadow) -> Self {
        blinc_core::Shadow {
            offset_x: shadow.offset_x,
            offset_y: shadow.offset_y,
            blur: shadow.blur,
            spread: shadow.spread,
            color: shadow.color,
        }
    }
}

/// Complete set of shadow tokens
#[derive(Clone, Debug)]
pub struct ShadowTokens {
    pub shadow_sm: Shadow,
    pub shadow_default: Shadow,
    pub shadow_md: Shadow,
    pub shadow_lg: Shadow,
    pub shadow_xl: Shadow,
    pub shadow_2xl: Shadow,
    pub shadow_inner: Shadow,
    pub shadow_none: Shadow,
}

impl ShadowTokens {
    /// Get shadow by token key
    pub fn get(&self, token: ShadowToken) -> &Shadow {
        match token {
            ShadowToken::Sm => &self.shadow_sm,
            ShadowToken::Default => &self.shadow_default,
            ShadowToken::Md => &self.shadow_md,
            ShadowToken::Lg => &self.shadow_lg,
            ShadowToken::Xl => &self.shadow_xl,
            ShadowToken::Xxl => &self.shadow_2xl,
            ShadowToken::Inner => &self.shadow_inner,
            ShadowToken::None => &self.shadow_none,
        }
    }

    /// Create shadow tokens for a light color scheme
    pub fn light() -> Self {
        let base_color = Color::BLACK;
        Self {
            shadow_sm: Shadow::new(0.0, 1.0, 2.0, 0.0, base_color.with_alpha(0.05)),
            shadow_default: Shadow::new(0.0, 1.0, 3.0, 0.0, base_color.with_alpha(0.1)),
            shadow_md: Shadow::new(0.0, 4.0, 6.0, -1.0, base_color.with_alpha(0.1)),
            shadow_lg: Shadow::new(0.0, 10.0, 15.0, -3.0, base_color.with_alpha(0.1)),
            shadow_xl: Shadow::new(0.0, 20.0, 25.0, -5.0, base_color.with_alpha(0.1)),
            shadow_2xl: Shadow::new(0.0, 25.0, 50.0, -12.0, base_color.with_alpha(0.25)),
            shadow_inner: Shadow::new(0.0, 2.0, 4.0, 0.0, base_color.with_alpha(0.05)),
            shadow_none: Shadow::none(),
        }
    }

    /// Create shadow tokens for a dark color scheme
    pub fn dark() -> Self {
        let base_color = Color::BLACK;
        Self {
            shadow_sm: Shadow::new(0.0, 1.0, 2.0, 0.0, base_color.with_alpha(0.2)),
            shadow_default: Shadow::new(0.0, 1.0, 3.0, 0.0, base_color.with_alpha(0.3)),
            shadow_md: Shadow::new(0.0, 4.0, 6.0, -1.0, base_color.with_alpha(0.3)),
            shadow_lg: Shadow::new(0.0, 10.0, 15.0, -3.0, base_color.with_alpha(0.3)),
            shadow_xl: Shadow::new(0.0, 20.0, 25.0, -5.0, base_color.with_alpha(0.3)),
            shadow_2xl: Shadow::new(0.0, 25.0, 50.0, -12.0, base_color.with_alpha(0.5)),
            shadow_inner: Shadow::new(0.0, 2.0, 4.0, 0.0, base_color.with_alpha(0.15)),
            shadow_none: Shadow::none(),
        }
    }

    /// Linear interpolation between two shadow token sets
    pub fn lerp(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            shadow_sm: Shadow::lerp(&from.shadow_sm, &to.shadow_sm, t),
            shadow_default: Shadow::lerp(&from.shadow_default, &to.shadow_default, t),
            shadow_md: Shadow::lerp(&from.shadow_md, &to.shadow_md, t),
            shadow_lg: Shadow::lerp(&from.shadow_lg, &to.shadow_lg, t),
            shadow_xl: Shadow::lerp(&from.shadow_xl, &to.shadow_xl, t),
            shadow_2xl: Shadow::lerp(&from.shadow_2xl, &to.shadow_2xl, t),
            shadow_inner: Shadow::lerp(&from.shadow_inner, &to.shadow_inner, t),
            shadow_none: Shadow::none(),
        }
    }
}

impl Default for ShadowTokens {
    fn default() -> Self {
        Self::light()
    }
}
