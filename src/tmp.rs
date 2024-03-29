use std::f64::consts::PI;

use crate::vector::Vector3;
use drawing_stuff::canvas::{Canvas, Draw};
use drawing_stuff::color::RGBA;

pub struct Camera {
    position: Vector3,
    direction: Vector3,
    horizontal: Vector3,
    vertical: Vector3,

    fov_horizontal: f64,
    fov_vertical: f64,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            direction: Vector3::unit_x(),
            horizontal: Vector3::unit_y(),
            vertical: Vector3::unit_z(),
            fov_horizontal: 1.57,
            fov_vertical: 1.57,
        }
    }

    pub fn position(&self) -> Vector3 {
        self.position
    }
    pub fn set_position(&mut self, pos: Vector3) {
        self.position = pos;
    }
    pub fn r#move(&mut self, mov: Vector3) {
        self.position += mov;
    }

    pub fn direction(&self) -> Vector3 {
        self.direction
    }
    pub fn horizontal(&self) -> Vector3 {
        self.horizontal
    }
    pub fn vertical(&self) -> Vector3 {
        self.vertical
    }

    pub fn rotate_horizontal(&mut self, angle: f64) {
        let rot_axis = self.vertical;

        self.direction = self.direction.rotate(rot_axis, angle);
        self.horizontal = self.vertical.rotate(rot_axis, angle);
    }
    pub fn rotate_vertical(&mut self, angle: f64) {
        let rot_axis = self.horizontal;

        self.direction = self.direction.rotate(rot_axis, angle);
        self.vertical = self.vertical.rotate(rot_axis, angle);
    }

    pub fn point_2d_normalized(&self, v: Vector3) -> (f64, f64) {
        let dir = v - self.position;

        let ang_hor = dir.angle_plane(self.horizontal);
        let ang_vert = -dir.angle_plane(self.vertical);

        let ang_hor = if dir.dot_product(self.direction) > 0.0 {
            ang_hor
        } else {
            if ang_hor > 0.0 {
                PI - ang_hor
            } else {
                -PI - ang_hor
            }
        };
        let ang_vert = if dir.dot_product(self.direction) > 0.0 {
            ang_vert
        } else {
            if ang_vert > 0.0 {
                PI - ang_vert
            } else {
                -PI - ang_vert
            }
        };

        println!("({}, {})", ang_hor, ang_vert);

        let norm_x = (ang_hor / self.fov_horizontal) + 0.5;
        let norm_y = (ang_vert / self.fov_vertical) + 0.5;

        (norm_x, norm_y)
    }
}

pub trait Project<T>
where
    T: Draw,
{
    fn project(self, camera: &Camera, canvas: &Canvas) -> T;
}

pub struct Line {
    pub start: Vector3,
    pub end: Vector3,
}
impl Project<LineProjection> for Line {
    fn project(self, camera: &Camera, canvas: &Canvas) -> LineProjection {
        let (start_rel_x, start_rel_y) = camera.point_2d_normalized(self.start);
        let (end_rel_x, end_rel_y) = camera.point_2d_normalized(self.end);

        let start_x = (start_rel_x * canvas.width() as f64) as isize;
        let start_y = (start_rel_y * canvas.height() as f64) as isize;

        let end_x = (end_rel_x * canvas.width() as f64) as isize;
        let end_y = (end_rel_y * canvas.height() as f64) as isize;

        LineProjection {
            start: (start_x, start_y),
            end: (end_x, end_y),
        }
    }
}

pub struct LineProjection {
    pub start: (isize, isize),
    pub end: (isize, isize),
}
impl Draw for LineProjection {
    fn draw(&self, canvas: &mut Canvas) {
        canvas.draw_line(
            self.start.0,
            self.start.1,
            self.end.0,
            self.end.1,
            RGBA {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        )
    }
}
