use std::ops::Range;

use crate::graphing::{CoordinateStyle, FunctionStyle, Graphing, PointStyle};
use drawing_stuff::canvas::{Canvas, Draw};
use drawing_stuff::color::{BLACK, RGBA};
use drawing_stuff::drawables::{Circle, Line};

/// Graph2D is used to compose a 2-dimensional graph and draw it to a `Canvas`.
pub struct Graph2D {
    /// The width of the drawing area.
    width: usize,
    /// The height of the drawing area.
    height: usize,

    /// The margin to the sides of the x-direction given in global drawing coordinates.
    x_margin: usize,
    /// The margin to the sides of the y-direction given in global drawing coordinates.
    y_margin: usize,

    /// The x-range of the local graphing coordinates.
    x_range: Range<f64>,
    /// The y-range of the local graphing coordinates.
    y_range: Range<f64>,

    drawing_buffer: Vec<Box<dyn Draw>>,
}

impl Graph2D {
    /// Creates an empty 2-dimensional graph.
    pub fn new(
        width: usize,
        height: usize,
        x_margin: usize,
        y_margin: usize,
        x_range: Range<f64>,
        y_range: Range<f64>,
    ) -> Self {
        Self {
            width,
            height,
            x_margin,
            y_margin,
            x_range,
            y_range,
            drawing_buffer: Vec::new(),
        }
    }

    /// Returns the width subtracting the margin from both sides.
    fn drawing_width(&self) -> usize {
        self.width - 2 * self.x_margin
    }

    /// Returns the height subtracting the margin from both sides.
    fn drawing_height(&self) -> usize {
        self.height - 2 * self.y_margin
    }

    /// Returns the length of the x-range
    fn x_range_len(&self) -> f64 {
        (self.x_range.end - self.x_range.start).abs()
    }

    /// Returns the length of the y-range
    fn y_range_len(&self) -> f64 {
        (self.y_range.end - self.y_range.start).abs()
    }

    /// Converts local graphing coordinates to global drawing coordinates.
    fn local_to_global(&self, local: (f64, f64)) -> (isize, isize) {
        let (lx, ly) = local;

        let gx = (((lx - self.x_range.start) / self.x_range_len()) * self.drawing_width() as f64)
            as isize
            + self.x_margin as isize;
        let gy = ((-(ly - self.y_range.end) / self.y_range_len()) * self.drawing_height() as f64)
            as isize
            + self.y_margin as isize;

        (gx, gy)
    }

    /// Returns the closest multiple of the base laying in the direction to 0.
    fn abs_floor_multiple(num: f64, base: f64) -> f64 {
        let multiple = (num.abs() / base.abs()).floor();
        match num >= 0.0 {
            true => base * multiple,
            false => -(base * multiple),
        }
    }
}

impl Graphing for Graph2D {
    type Point = (f64, f64);
    type Function = fn(f64) -> f64;

    fn add_cartesian(&mut self, style: CoordinateStyle) {
        let axes_color = style.axes_color.unwrap_or(BLACK);
        let tick_spacing = style.tick_spacing.unwrap_or(1.0).abs();
        let tick_size = style.tick_size.unwrap_or(10.0);
        let tick_color = style.tick_color.unwrap_or(BLACK);
        let grid = style.grid.unwrap_or(true);
        let grid_color = style.grid_color.unwrap_or(RGBA::new(0, 0, 0, 32));
        let light_grid = style.light_grid.unwrap_or(true);
        let light_grid_density = style.light_grid_density.unwrap_or(5);
        let light_grid_color = style.light_grid_color.unwrap_or(RGBA::new(0, 0, 0, 8));

        let y_ax_x = if self.x_range.contains(&0.0) {
            0.0
        } else if self.x_range.start.abs() < self.x_range.end.abs() {
            self.x_range.start
        } else {
            self.x_range.end
        };

        let x_ax_y = if self.y_range.contains(&0.0) {
            0.0
        } else if self.y_range.start.abs() < self.y_range.end.abs() {
            self.y_range.start
        } else {
            self.y_range.end
        };

        let x_ax = Line {
            end1: self.local_to_global((self.x_range.start, x_ax_y)),
            end2: self.local_to_global((self.x_range.end, x_ax_y)),
            color: axes_color,
        };
        self.drawing_buffer.push(Box::new(x_ax));

        let y_ax = Line {
            end1: self.local_to_global((y_ax_x, self.y_range.start)),
            end2: self.local_to_global((y_ax_x, self.y_range.end)),
            color: axes_color,
        };
        self.drawing_buffer.push(Box::new(y_ax));

        let x_range_min = f64::min(self.x_range.start, self.x_range.end);
        let x_range_max = f64::max(self.x_range.start, self.x_range.end);
        let x_ticks_start = Self::abs_floor_multiple(x_range_min, tick_spacing);

        let mut x_pos = x_ticks_start;
        while x_pos <= x_range_max {
            let pos = self.local_to_global((x_pos, x_ax_y));

            let tick = Line {
                end1: (pos.0, pos.1 + (tick_size / 2.0) as isize),
                end2: (pos.0, pos.1 - (tick_size / 2.0) as isize),
                color: tick_color,
            };
            self.drawing_buffer.push(Box::new(tick));

            if grid {
                let grid_line = Line {
                    end1: self.local_to_global((x_pos, self.y_range.start)),
                    end2: self.local_to_global((x_pos, self.y_range.end)),
                    color: grid_color,
                };
                self.drawing_buffer.push(Box::new(grid_line));
            }

            if light_grid && x_pos + tick_spacing <= x_range_max {
                for i in 1..light_grid_density {
                    let x_pos = x_pos + i as f64 * (tick_spacing / light_grid_density as f64);

                    let grid_line = Line {
                        end1: self.local_to_global((x_pos, self.y_range.start)),
                        end2: self.local_to_global((x_pos, self.y_range.end)),
                        color: light_grid_color,
                    };
                    self.drawing_buffer.push(Box::new(grid_line));
                }
            }

            x_pos += tick_spacing;
        }

        let y_range_min = f64::min(self.y_range.start, self.y_range.end);
        let y_range_max = f64::max(self.y_range.start, self.y_range.end);
        let y_ticks_start = Self::abs_floor_multiple(y_range_min, tick_spacing);

        let mut y_pos = y_ticks_start;
        while y_pos <= y_range_max {
            let pos = self.local_to_global((y_ax_x, y_pos));

            let tick = Line {
                end1: (pos.0 + (tick_size / 2.0) as isize, pos.1),
                end2: (pos.0 - (tick_size / 2.0) as isize, pos.1),
                color: tick_color,
            };
            self.drawing_buffer.push(Box::new(tick));

            if grid {
                let grid_line = Line {
                    end1: self.local_to_global((self.x_range.start, y_pos)),
                    end2: self.local_to_global((self.x_range.end, y_pos)),
                    color: grid_color,
                };
                self.drawing_buffer.push(Box::new(grid_line));
            }

            if light_grid && y_pos + tick_spacing <= y_range_max {
                for i in 1..light_grid_density {
                    let y_pos = y_pos + i as f64 * (tick_spacing / light_grid_density as f64);

                    let grid_line = Line {
                        end1: self.local_to_global((self.x_range.start, y_pos)),
                        end2: self.local_to_global((self.x_range.end, y_pos)),
                        color: light_grid_color,
                    };
                    self.drawing_buffer.push(Box::new(grid_line));
                }
            }

            y_pos += tick_spacing;
        }
    }

    fn add_point(&mut self, point: Self::Point, style: PointStyle) {
        let solid = style.solid.unwrap_or(true);
        let color = style.color.unwrap_or(BLACK);
        let radius = style.radius.unwrap_or(3.0);

        let point = Circle {
            center: self.local_to_global(point),
            radius: radius as usize,
            solid,
            color,
        };

        self.drawing_buffer.push(Box::new(point));
    }

    fn add_function(&mut self, function: Self::Function, style: FunctionStyle) {
        let resolution = style.resolution.unwrap_or(1000);
        let color = style.color.unwrap_or(BLACK);

        if resolution == 0 {
            return;
        }

        let mut samples = Vec::new();
        for i in 0..resolution {
            let current = i as f64 / (resolution - 1) as f64;

            let x_range_min = f64::min(self.x_range.start, self.x_range.end);
            let x = x_range_min + current * self.x_range_len();

            let y = function(x);

            samples.push(self.local_to_global((x, y)));
        }

        for i in 1..samples.len() {
            let line = Line {
                end1: samples[i - 1],
                end2: samples[i],
                color,
            };

            self.drawing_buffer.push(Box::new(line));
        }
    }
}

impl Draw for Graph2D {
    fn draw(&self, canvas: &mut Canvas) {
        for drawable in self.drawing_buffer.iter() {
            drawable.draw(canvas);
        }
    }
}
