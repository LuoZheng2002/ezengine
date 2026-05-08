use crate::{color::Color, geometry::Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientPoint {
    pub stop: f32,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Brush {
    Solid(Color),
    LinearGradient {
        from: Point,
        to: Point,
        stops: Vec<GradientPoint>,
    },
    RadialGradient {
        center: Point,
        stops: Vec<GradientPoint>,
    },
}

impl Brush {
    pub fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    pub fn solid_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::Solid(Color::rgb(r, g, b))
    }

    pub fn to_color(&self) -> Color {
        match self {
            Self::Solid(color) => *color,
            Self::LinearGradient { stops, .. } | Self::RadialGradient { stops, .. } => {
                stops.first().map(|point| point.color).unwrap_or(Color::WHITE)
            }
        }
    }
}

impl Default for Brush {
    fn default() -> Self {
        Self::Solid(Color::WHITE)
    }
}
