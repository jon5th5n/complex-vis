use crate::vector::{Vector2, Vector3};
use drawing_stuff::color::RGBA;

/// Trait for objects defined by signed distance functions.
pub trait SDF {
    type Point;

    /// Returns the signed distance.
    fn sdf(&self, p: Self::Point) -> f64;

    // Returns the color.
    fn color(&self) -> RGBA;
}

pub struct Line2D {
    pub end1: Vector2,
    pub end2: Vector2,

    pub width: f64,

    pub color: RGBA,
}

impl SDF for Line2D {
    type Point = Vector2;

    fn sdf(&self, p: Self::Point) -> f64 {
        let pe1 = p - self.end1;
        let e2e1 = self.end2 - self.end1;
        let h = (pe1.dot_product(e2e1) / e2e1.dot_product(e2e1)).clamp(0.0, 1.0);
        (pe1 - e2e1 * h).length() - (self.width / 2.0)
    }

    fn color(&self) -> RGBA {
        self.color
    }
}

pub struct LinePath2D {
    pub points: Vec<Vector2>,

    pub width: f64,

    pub color: RGBA,
}

impl SDF for LinePath2D {
    type Point = Vector2;

    fn sdf(&self, p: Self::Point) -> f64 {
        if self.points.len() == 0 {
            return f64::INFINITY;
        }

        if self.points.len() == 1 {
            return (self.points[0] - p).length() - (self.width / 2.0);
        }

        let mut min = f64::INFINITY;
        for i in 1..self.points.len() {
            let end1 = self.points[i - 1];
            let end2 = self.points[i];

            let pe1 = p - end1;
            let e2e1 = end2 - end1;
            let h = (pe1.dot_product(e2e1) / e2e1.dot_product(e2e1)).clamp(0.0, 1.0);
            let dist = (pe1 - e2e1 * h).length();

            if dist < min {
                min = dist;
            }
        }

        min - (self.width / 2.0)
    }

    fn color(&self) -> RGBA {
        self.color
    }
}

pub struct Circle2D {
    pub center: Vector2,
    pub radius: f64,

    pub color: RGBA,
}

impl SDF for Circle2D {
    type Point = Vector2;

    fn sdf(&self, p: Self::Point) -> f64 {
        (self.center - p).length() - self.radius
    }

    fn color(&self) -> RGBA {
        self.color
    }
}
