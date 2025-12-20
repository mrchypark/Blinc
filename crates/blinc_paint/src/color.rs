//! Color types and utilities

/// RGBA color with f32 components (0.0 to 1.0)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create from u8 components (0-255)
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create from hex value (0xRRGGBB or 0xRRGGBBAA)
    pub fn from_hex(hex: u32) -> Self {
        if hex > 0xFFFFFF {
            // 0xRRGGBBAA format
            Self::from_rgba8(
                ((hex >> 24) & 0xFF) as u8,
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
            )
        } else {
            // 0xRRGGBB format
            Self::from_rgba8(
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
                255,
            )
        }
    }

    /// Create a grayscale color
    pub fn gray(value: f32) -> Self {
        Self::rgb(value, value, value)
    }

    /// Set alpha and return new color
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }

    /// Lighten the color
    pub fn lighten(self, amount: f32) -> Self {
        Self {
            r: (self.r + amount).min(1.0),
            g: (self.g + amount).min(1.0),
            b: (self.b + amount).min(1.0),
            a: self.a,
        }
    }

    /// Darken the color
    pub fn darken(self, amount: f32) -> Self {
        Self {
            r: (self.r - amount).max(0.0),
            g: (self.g - amount).max(0.0),
            b: (self.b - amount).max(0.0),
            a: self.a,
        }
    }

    /// Convert to u8 array [r, g, b, a]
    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }
}
