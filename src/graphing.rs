use drawing_stuff::color::RGBA;

#[derive(Debug, Default)]
/// Holds style settings which describe the look of a coordinate system.
/// Setting the Options to None will give you the default look determined by the backend.
pub struct CoordinateStyle {
    pub axes_color: Option<RGBA>,

    pub tick_spacing: Option<f64>,
    pub tick_size: Option<f64>,
    pub tick_color: Option<RGBA>,

    pub grid: Option<bool>,
    pub grid_color: Option<RGBA>,

    pub light_grid: Option<bool>,
    pub light_grid_density: Option<u32>,
    pub light_grid_color: Option<RGBA>,
}

impl CoordinateStyle {
    pub fn axes_color(mut self, color: RGBA) -> Self {
        self.axes_color = Some(color);
        self
    }

    pub fn tick_spacing(mut self, spacing: f64) -> Self {
        self.tick_spacing = Some(spacing);
        self
    }

    pub fn tick_size(mut self, size: f64) -> Self {
        self.tick_size = Some(size);
        self
    }

    pub fn tick_color(mut self, color: RGBA) -> Self {
        self.tick_color = Some(color);
        self
    }

    pub fn grid(mut self, b: bool) -> Self {
        self.grid = Some(b);
        self
    }

    pub fn grid_color(mut self, color: RGBA) -> Self {
        self.grid_color = Some(color);
        self
    }

    pub fn light_grid(mut self, b: bool) -> Self {
        self.light_grid = Some(b);
        self
    }

    pub fn light_grid_denisty(mut self, density: u32) -> Self {
        self.light_grid_density = Some(density);
        self
    }

    pub fn light_grid_color(mut self, color: RGBA) -> Self {
        self.light_grid_color = Some(color);
        self
    }
}

#[derive(Debug, Default)]
/// Holds style settings which describe the look of a point.
/// Setting the Options to None will give you the default look determined by the backend.
pub struct PointStyle {
    pub solid: Option<bool>,
    pub color: Option<RGBA>,
    pub radius: Option<f64>,
}

impl PointStyle {
    pub fn solid(mut self, b: bool) -> Self {
        self.solid = Some(b);
        self
    }

    pub fn color(mut self, color: RGBA) -> Self {
        self.color = Some(color);
        self
    }

    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = Some(radius);
        self
    }
}

#[derive(Debug, Default)]
/// Holds style settings which describe the look of the graph of a function.
/// Setting the Options to None will give you the default look determined by the backend.
pub struct FunctionStyle {
    pub resolution: Option<u32>,
    pub thickness: Option<f32>,
    pub color: Option<RGBA>,
}

impl FunctionStyle {
    pub fn resolution(mut self, res: u32) -> Self {
        self.resolution = Some(res);
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = Some(thickness);
        self
    }

    pub fn color(mut self, color: RGBA) -> Self {
        self.color = Some(color);
        self
    }
}

/// General functions needed for a graphing backend.
pub trait Graphing {
    /// The type representing a single point in the coordinate system.
    type Point;

    /// The type representing a mathematical function.
    type Function;

    /// Adds a cartesian coordinate system to the drawing pipeline
    fn add_cartesian(&mut self, style: CoordinateStyle);

    /// Adds point to the drawing pipeline
    fn add_point(&mut self, point: Self::Point, style: PointStyle);

    /// Adds a graph of a function to the drawing pipeline
    fn add_function(&mut self, function: Self::Function, style: FunctionStyle);
}
