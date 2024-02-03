use crate::canvas::{Canvas, Draw};
use crate::color::RGBA;

#[derive(Debug)]
pub struct Line {
    pub end1: (isize, isize),
    pub end2: (isize, isize),

    pub color: RGBA,
}

impl Draw for Line {
    fn draw(&self, canvas: &mut Canvas) {
        canvas.draw_line(
            self.end1.0,
            self.end1.1,
            self.end2.0,
            self.end2.1,
            self.color,
        );
    }
}

#[derive(Debug)]
pub struct Circle {
    pub center: (isize, isize),

    pub radius: usize,
    pub solid: bool,
    pub color: RGBA,
}

impl Draw for Circle {
    fn draw(&self, canvas: &mut Canvas) {
        println!("{:?}", self);
        match self.solid {
            true => canvas.draw_circle_solid(self.center.0, self.center.1, self.radius, self.color),
            false => canvas.draw_circle(self.center.0, self.center.1, self.radius, self.color),
        }
    }
}
