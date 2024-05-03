use std::ops::Range;

use crate::graphing::{CoordinateStyle, FunctionStyle, Graphing, PointStyle};
use drawing_stuff::canvas::{Canvas, Draw};
use drawing_stuff::drawables::{Circle, Line};
use drawing_stuff::rgba::{BLACK, RGBA};

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

    drawing_buffer: Vec<Box<dyn Draw<RGBA>>>,
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

    /// Clamps the specified coordinates of a line into the graphing area.
    /// Returns (-1, -1, -1, -1) if the line is not visible.
    fn clamp_line_coords(
        &self,
        x1: isize,
        y1: isize,
        x2: isize,
        y2: isize,
    ) -> (isize, isize, isize, isize) {
        let x_min = self.x_margin as isize;
        let y_min = self.y_margin as isize;
        let x_max = self.width as isize - self.x_margin as isize;
        let y_max = self.height as isize - self.y_margin as isize;

        let p1_inside = x1 >= x_min && x1 < x_max && y1 >= y_min && y1 < y_max;
        let p2_inside = x2 >= x_min && x2 < x_max && y2 >= y_min && y2 < y_max;

        if p1_inside && p2_inside {
            return (x1, y1, x2, y2);
        }

        let dx = x2 - x1;
        let dy = y2 - y1;

        if dx == 0 {
            let s_y_min = (x1, y_min);
            let s_y_max = (x1, y_max);

            let (x1, y1) = match p1_inside {
                true => (x1, y1),
                false => {
                    if y1 < y_min {
                        s_y_min
                    } else {
                        s_y_max
                    }
                }
            };
            let (x2, y2) = match p2_inside {
                true => (x2, y2),
                false => {
                    if y2 < y_min {
                        s_y_min
                    } else {
                        s_y_max
                    }
                }
            };

            if x1 == x2 && y1 == y2 {
                return (-1, -1, -1, -1);
            }

            return (x1, y1, x2, y2);
        }

        let m = dy as f32 / dx as f32;
        let c = y1 as f32 - m * x1 as f32;

        let s_x_min = (x_min as f32, c + m * x_min as f32);
        let s_x_max = (x_max as f32, c + m * x_max as f32);
        let s_y_min = ((y_min as f32 - c) / m, y_min as f32);
        let s_y_max = ((y_max as f32 - c) / m, y_max as f32);

        let s_x_min = match s_x_min.1 >= y_min as f32 && s_x_min.1 < y_max as f32 {
            true => Some(s_x_min),
            false => None,
        };
        let s_x_max = match s_x_max.1 >= y_min as f32 && s_x_max.1 < y_max as f32 {
            true => Some(s_x_max),
            false => None,
        };

        let s_y_min = match s_y_min.0 >= x_min as f32 && s_y_min.0 < x_max as f32 {
            true => Some(s_y_min),
            false => None,
        };
        let s_y_max = match s_y_max.0 >= x_min as f32 && s_y_max.0 < x_max as f32 {
            true => Some(s_y_max),
            false => None,
        };

        let valid_intersects = [s_x_min, s_x_max, s_y_min, s_y_max]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if valid_intersects.len() < 2 {
            return (-1, -1, -1, -1);
        }

        let p1 = valid_intersects[0];
        let p2 = valid_intersects[1];

        let p1 = (p1.0.round() as isize, p1.1.round() as isize);
        let p2 = (p2.0.round() as isize, p2.1.round() as isize);

        let (x1, y1) = if p1_inside {
            (x1, y1)
        } else {
            let dx_p1 = p1.0 - x1;
            let dy_p1 = p1.1 - y1;
            let sqr_dist_p1 = dx_p1 * dx_p1 + dy_p1 * dy_p1;

            let dx_p2 = p2.0 - x1;
            let dy_p2 = p2.1 - y1;
            let sqr_dist_p2 = dx_p2 * dx_p2 + dy_p2 * dy_p2;

            if sqr_dist_p1 < sqr_dist_p2 {
                p1
            } else {
                p2
            }
        };
        let (x2, y2) = if p2_inside {
            (x2, y2)
        } else {
            let dx_p1 = p1.0 - x2;
            let dy_p1 = p1.1 - y2;
            let sqr_dist_p1 = dx_p1 * dx_p1 + dy_p1 * dy_p1;

            let dx_p2 = p2.0 - x2;
            let dy_p2 = p2.1 - y2;
            let sqr_dist_p2 = dx_p2 * dx_p2 + dy_p2 * dy_p2;

            if sqr_dist_p1 < sqr_dist_p2 {
                p1
            } else {
                p2
            }
        };

        if x1 == x2 && y1 == y2 {
            return (-1, -1, -1, -1);
        }

        (x1, y1, x2, y2)
    }
}

impl Graphing for Graph2D {
    type Point = (f64, f64);
    type Function = Box<dyn Fn(f64) -> f64>;

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

        let x_ax = Line::<RGBA> {
            end1: self.local_to_global((self.x_range.start, x_ax_y)),
            end2: self.local_to_global((self.x_range.end, x_ax_y)),
            width: 1,
            capped: false,
            pixel: axes_color,
        };
        self.drawing_buffer.push(Box::new(x_ax));

        let y_ax = Line::<RGBA> {
            end1: self.local_to_global((y_ax_x, self.y_range.start)),
            end2: self.local_to_global((y_ax_x, self.y_range.end)),
            width: 1,
            capped: false,
            pixel: axes_color,
        };
        self.drawing_buffer.push(Box::new(y_ax));

        let x_range_min = f64::min(self.x_range.start, self.x_range.end);
        let x_range_max = f64::max(self.x_range.start, self.x_range.end);
        let x_ticks_start = Self::abs_floor_multiple(x_range_min, tick_spacing);

        let mut x_pos = x_ticks_start;
        while x_pos <= x_range_max {
            let pos = self.local_to_global((x_pos, x_ax_y));

            let tick = Line::<RGBA> {
                end1: (pos.0, pos.1 + (tick_size / 2.0) as isize),
                end2: (pos.0, pos.1 - (tick_size / 2.0) as isize),
                width: 1,
                capped: false,
                pixel: tick_color,
            };
            self.drawing_buffer.push(Box::new(tick));

            if grid {
                let grid_line = Line::<RGBA> {
                    end1: self.local_to_global((x_pos, self.y_range.start)),
                    end2: self.local_to_global((x_pos, self.y_range.end)),
                    width: 1,
                    capped: false,
                    pixel: grid_color,
                };
                self.drawing_buffer.push(Box::new(grid_line));
            }

            if light_grid && x_pos + tick_spacing <= x_range_max {
                for i in 1..light_grid_density {
                    let x_pos = x_pos + i as f64 * (tick_spacing / light_grid_density as f64);

                    let grid_line = Line::<RGBA> {
                        end1: self.local_to_global((x_pos, self.y_range.start)),
                        end2: self.local_to_global((x_pos, self.y_range.end)),
                        width: 1,
                        capped: false,
                        pixel: light_grid_color,
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

            let tick = Line::<RGBA> {
                end1: (pos.0 + (tick_size / 2.0) as isize, pos.1),
                end2: (pos.0 - (tick_size / 2.0) as isize, pos.1),
                width: 1,
                capped: false,
                pixel: tick_color,
            };
            self.drawing_buffer.push(Box::new(tick));

            if grid {
                let grid_line = Line::<RGBA> {
                    end1: self.local_to_global((self.x_range.start, y_pos)),
                    end2: self.local_to_global((self.x_range.end, y_pos)),
                    width: 1,
                    capped: false,
                    pixel: grid_color,
                };
                self.drawing_buffer.push(Box::new(grid_line));
            }

            if light_grid && y_pos + tick_spacing <= y_range_max {
                for i in 1..light_grid_density {
                    let y_pos = y_pos + i as f64 * (tick_spacing / light_grid_density as f64);

                    let grid_line = Line::<RGBA> {
                        end1: self.local_to_global((self.x_range.start, y_pos)),
                        end2: self.local_to_global((self.x_range.end, y_pos)),
                        width: 1,
                        capped: false,
                        pixel: light_grid_color,
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

        let point = Circle::<RGBA> {
            center: self.local_to_global(point),
            radius: radius as u32,
            solid,
            pixel: color,
        };

        self.drawing_buffer.push(Box::new(point));
    }

    fn add_function(&mut self, function: Self::Function, style: FunctionStyle) {
        let resolution = style.resolution.unwrap_or(1000);
        let thickness = style.thickness.unwrap_or(1);
        let color = style.color.unwrap_or(BLACK);

        if resolution == 0 || thickness == 0 {
            return;
        }

        let thickness = if thickness == 1 {
            thickness
        } else {
            thickness + (thickness % 2)
        };

        let mut samples = Vec::new();
        for i in 0..resolution {
            let current = i as f64 / (resolution - 1) as f64;

            let x_range_min = f64::min(self.x_range.start, self.x_range.end);
            let x = x_range_min + current * self.x_range_len();

            let y = function(x);

            samples.push(self.local_to_global((x, y)));
        }

        for i in 1..samples.len() {
            let sample_i1_outside = (samples[i - 1].0 < 0
                || samples[i - 1].0 >= self.width as isize)
                || (samples[i - 1].1 < 0 || samples[i - 1].1 >= self.height as isize);
            let sample_i_putside = (samples[i].0 < 0 || samples[i].0 >= self.width as isize)
                || (samples[i].1 < 0 || samples[i].1 >= self.height as isize);

            if sample_i1_outside && sample_i_putside {
                continue;
            }

            let end1 = samples[i - 1];
            let end2 = samples[i];

            let (x1, y1, x2, y2) = self.clamp_line_coords(end1.0, end1.1, end2.0, end2.1);

            if x1 == -1 && y1 == -1 && y2 == -1 && x2 == -1 {
                continue;
            }

            let line = Line::<RGBA> {
                end1: (x1, y1),
                end2: (x2, y2),
                width: thickness,
                capped: true,
                pixel: color,
            };

            self.drawing_buffer.push(Box::new(line));
        }
    }
}

impl Draw<RGBA> for Graph2D {
    fn draw(&self, canvas: &mut Canvas<RGBA>) {
        for drawable in self.drawing_buffer.iter() {
            drawable.draw(canvas);
        }
    }
}
