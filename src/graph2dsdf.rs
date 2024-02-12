use std::ops::Range;

use crate::graphing::Graphing;
use crate::sdf::{Circle2D, Line2D, LinePath2D, SDF};
use crate::vector::Vector2;
use drawing_stuff::canvas::{Canvas, Draw};
use drawing_stuff::color::{BLACK, RGB, RGBA};

pub struct Graph2DSDF {
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

    sdf_buffer: Vec<Box<dyn SDF<Point = Vector2>>>,
}

impl Graph2DSDF {
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
            sdf_buffer: Vec::new(),
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
    fn local_to_global(&self, local: (f64, f64)) -> (f64, f64) {
        let (lx, ly) = local;

        let gx = (((lx - self.x_range.start) / self.x_range_len()) * self.drawing_width() as f64)
            + self.x_margin as f64;
        let gy = ((-(ly - self.y_range.end) / self.y_range_len()) * self.drawing_height() as f64)
            + self.y_margin as f64;

        (gx, gy)
    }

    // /// Returns the closest multiple of the base laying in the direction to 0.
    // fn abs_floor_multiple(num: f64, base: f64) -> f64 {
    //     let multiple = (num.abs() / base.abs()).floor();
    //     match num >= 0.0 {
    //         true => base * multiple,
    //         false => -(base * multiple),
    //     }
    // }
}

impl Graphing for Graph2DSDF {
    type Point = (f64, f64);
    type Function = fn(f64) -> f64;

    fn add_cartesian(&mut self, style: crate::CoordinateStyle) {
        todo!()
    }

    fn add_point(&mut self, point: Self::Point, style: crate::PointStyle) {
        let radius = style.radius.unwrap_or(3.0);
        let solid = style.solid.unwrap_or(true);
        let color = style.color.unwrap_or(BLACK);

        let circle = Circle2D {
            center: Vector2::from_point(self.local_to_global(point)),
            radius,
            color,
        };

        self.sdf_buffer.push(Box::new(circle));
    }

    fn add_function(&mut self, function: Self::Function, style: crate::FunctionStyle) {
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

            samples.push(Vector2::from_point(self.local_to_global((x, y))));
        }

        let line_path = LinePath2D {
            points: samples,
            width: 5.0,
            color,
        };

        self.sdf_buffer.push(Box::new(line_path));
    }
}

impl Draw for Graph2DSDF {
    fn draw(&self, canvas: &mut Canvas) {
        if self.sdf_buffer.len() == 0 {
            return;
        }

        let mut mask = Canvas::new(self.width, self.height);

        for x in 0..self.width {
            for y in 0..self.height {
                if mask.get(x, y).unwrap()
                    == &(RGB {
                        r: 255,
                        g: 255,
                        b: 255,
                    })
                {
                    continue;
                }

                let p = Vector2::new(x as f64, y as f64);

                let mut min_dist = self.sdf_buffer[0].sdf(p);
                let mut color = self.sdf_buffer[0].color();
                for sdf in self.sdf_buffer.iter().skip(1) {
                    let dist = sdf.sdf(p);
                    if dist < min_dist {
                        min_dist = dist;
                        color = sdf.color();
                    }
                }

                if min_dist <= 0.0 {
                    canvas.set(x, y, color.to_rgb().0);
                } else {
                    mask.draw_circle_solid(
                        x as isize,
                        y as isize,
                        min_dist.floor() as usize,
                        RGBA {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 255,
                        },
                    );
                }
            }
        }
    }
}
