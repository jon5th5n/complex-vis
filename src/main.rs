use std::fmt::Debug;

use minifb::{Key, Window, WindowOptions};

mod complex;
use complex::*;

mod color;
use color::{RGB, RGBA};

mod colors;
use colors::*;

mod canvas;
use canvas::{Canvas, Draw};

mod drawables;
use drawables::*;

mod graphing;
use graphing::*;

mod graph2d;
use graph2d::*;

mod vector;
use vector::Vector;

mod quaternion;
use quaternion::Quaternion;

// mod tmp;
// use tmp::*;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;

fn main() {
    println!("hello world");

    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    let mut window = Window::new("Complex Stuff", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        canvas.fill(RGB {
            r: 255,
            g: 255,
            b: 255,
        });

        let mut graph = Graph2D::new(WIDTH, HEIGHT, WIDTH / 20, HEIGHT / 5, -10.5..5.0, -2.5..5.0);

        graph.add_cartesian(CoordinateStyle::default());
        graph.add_point((2.1, 0.4), PointStyle::default().color(RED));
        graph.add_function(|x| x.sin(), FunctionStyle::default().color(GREEN));
        graph.add_function(|x| x.cos(), FunctionStyle::default().color(BLUE));

        graph.draw(&mut canvas);

        window
            .update_with_buffer(&canvas.buffer_u32(), WIDTH, HEIGHT)
            .unwrap();
    }
}
