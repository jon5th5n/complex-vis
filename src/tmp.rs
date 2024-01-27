mod vector;
use vector::Vector;

mod quaternion;
use quaternion::Quaternion;

pub struct Camera {
    position: Vector,
    direction: Vector,

    horizontal: Vector,
    vertical: Vector,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vector::zero(),
            direction: Vector::unit_x(),
            horizontal: Vector::unit_y(),
            vertical: Vector::unit_z(),
        }
    }

    pub fn position(&self) -> Vector {
        self.position
    }
    pub fn set_position(&mut self, pos: Vector) {
        self.position = pos;
    }
    pub fn r#move(&mut self, mov: Vector) {
        self.position += mov;
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
}

pub trait Project<T>
where
    T: Draw,
{
    fn project(self) -> T;
}

pub trait Draw {
    fn draw(&self, canvas: &mut Vec<u32>);
}

struct Line {
    start: Vector,
    end: Vector,
}
impl Project<LineProjection> for Line {
    fn project(self) -> LineProjection {
        LineProjection {
            start: (self.start.x, self.start.y),
            end: (self.end.x, self.end.y),
        }
    }
}

struct LineProjection {
    start: (f64, f64),
    end: (f64, f64),
}
impl Draw for LineProjection {
    fn draw(&self, canvas: &mut Vec<u32>) {}
}
