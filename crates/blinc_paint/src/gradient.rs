//! Gradient fills

use crate::color::Color;
use crate::path::Point;

/// A gradient stop
#[derive(Clone, Copy, Debug)]
pub struct GradientStop {
    pub offset: f32, // 0.0 to 1.0
    pub color: Color,
}

/// Gradient type
#[derive(Clone, Debug)]
pub enum Gradient {
    Linear {
        start: Point,
        end: Point,
        stops: Vec<GradientStop>,
    },
    Radial {
        center: Point,
        radius: f32,
        stops: Vec<GradientStop>,
    },
    Conic {
        center: Point,
        angle: f32,
        stops: Vec<GradientStop>,
    },
}

impl Gradient {
    /// Create a simple linear gradient between two colors
    pub fn linear_simple(start: Point, end: Point, from: Color, to: Color) -> Self {
        Gradient::Linear {
            start,
            end,
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: from,
                },
                GradientStop {
                    offset: 1.0,
                    color: to,
                },
            ],
        }
    }

    /// Create a simple radial gradient between two colors
    pub fn radial_simple(center: Point, radius: f32, from: Color, to: Color) -> Self {
        Gradient::Radial {
            center,
            radius,
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: from,
                },
                GradientStop {
                    offset: 1.0,
                    color: to,
                },
            ],
        }
    }
}
