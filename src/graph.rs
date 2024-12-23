use wgpu_text::glyph_brush::ab_glyph::FontArc;

use crate::decimal_math::Decimal;
use crate::{color::RGBA, gpuview::Font};

/// Structure respresenting the graph of a function.
///
/// `I`: Input;
/// `P`: Parameter;
/// `O`: Output;
#[derive(Debug, Clone)]
pub struct FunctionGraph<I, P, O> {
    pub function: fn(I, &P) -> O,
    pub style: GraphStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct GraphStyle {
    pub color: RGBA,
    pub thickness: f32,
}

impl Default for GraphStyle {
    fn default() -> Self {
        Self {
            color: RGBA::BLACK,
            thickness: Thickness::MEDIUM,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnviromentStyle {
    // pub background_color: RGBA,
    pub x: DimensionStyle,
    pub y: DimensionStyle,
    pub text: Option<TextStyle>,
}

impl Default for EnviromentStyle {
    fn default() -> Self {
        Self {
            x: DimensionStyle::default(),
            y: DimensionStyle::default(),
            text: Some(TextStyle::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DimensionStyle {
    pub spacing: GridSpacing,
    pub axis: Option<AxisStyle>,
    pub tick: Option<TickStyle>,
    pub subtick: Option<TickStyle>,
    pub grid: Option<GridStyle>,
    pub subgrid: Option<GridStyle>,
}

impl Default for DimensionStyle {
    fn default() -> Self {
        Self {
            spacing: GridSpacing::default(),
            axis: Some(AxisStyle::default()),
            tick: None,
            subtick: None,
            grid: Some(GridStyle::default()),
            subgrid: Some(GridStyle {
                color: RGBA::grey(240),
                thickness: Thickness::EXTRATHIN,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GridSpacing {
    Dynamic { steps: u32, substeps: u32 },
    Fixed { spacing: Decimal, substeps: u32 },
}

impl Default for GridSpacing {
    fn default() -> Self {
        Self::Dynamic {
            steps: 10,
            substeps: 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AxisStyle {
    pub color: RGBA,
    pub thickness: f32,
}

impl Default for AxisStyle {
    fn default() -> Self {
        Self {
            color: RGBA::BLACK,
            thickness: Thickness::THIN,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TickStyle {
    pub color: RGBA,
    pub length: f32,
    pub thickness: f32,
}

impl Default for TickStyle {
    fn default() -> Self {
        Self {
            color: RGBA::BLACK,
            length: 0.015,
            thickness: Thickness::MEDIUM,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridStyle {
    pub color: RGBA,
    pub thickness: f32,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            color: RGBA::grey(200),
            thickness: Thickness::THIN,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub size: f32,
    pub font: Font,
    /// Maximum number of digits before switching to scientific notation
    pub max_digits: u32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 32.0,
            font: Font {
                name: "Default".to_string(),
                font: FontArc::try_from_vec(std::fs::read("fonts/DejaVuSans.ttf").unwrap())
                    .unwrap(),
            },
            max_digits: 4,
        }
    }
}

pub struct Thickness;
impl Thickness {
    pub const EXTRATHIN: f32 = 0.001;
    pub const THIN: f32 = 0.0025;
    pub const MEDIUM: f32 = 0.005;
    pub const BOLD: f32 = 0.0075;
    pub const EXTRABOLD: f32 = 0.01;
}
