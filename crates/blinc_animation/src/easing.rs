//! Easing functions for animations

/// Easing function type
#[derive(Clone, Copy, Debug, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// Apply the easing function to a progress value (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::EaseInQuart => t * t * t * t,
            Easing::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Easing::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }
            Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier_ease(t, *x1, *y1, *x2, *y2),
        }
    }
}

/// Cubic bezier easing calculation
fn cubic_bezier_ease(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Newton-Raphson iteration to find t for given x
    let mut guess = t;
    for _ in 0..8 {
        let x = cubic_bezier_sample(guess, x1, x2) - t;
        if x.abs() < 0.001 {
            break;
        }
        let dx = cubic_bezier_slope(guess, x1, x2);
        if dx.abs() < 0.000001 {
            break;
        }
        guess -= x / dx;
    }
    cubic_bezier_sample(guess, y1, y2)
}

fn cubic_bezier_sample(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

fn cubic_bezier_slope(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let mt = 1.0 - t;

    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t2 * (1.0 - p2)
}
