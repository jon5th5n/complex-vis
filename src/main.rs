use std::time::Instant;

use minifb::{Key, Window, WindowOptions};

use complex_stuff::*;

use drawing_stuff::canvas::{Canvas, Draw};
use drawing_stuff::color::*;
use drawing_stuff::drawables::*;

mod graphing;
use graphing::*;

mod graph2d;
use graph2d::*;

mod vector;
use vector::Vector3;

mod quaternion;
use quaternion::Quaternion;

mod tmp;
use tmp::*;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;

fn main() {
    println!("hello world");

    let mut last_tick = Instant::now();

    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    let mut window = Window::new("Complex Stuff", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut counter = 0.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta = now - last_tick;
        println!("{}", delta.as_millis());
        last_tick = now;

        canvas.fill(RGB {
            r: 255,
            g: 255,
            b: 255,
        });

        let mut graph = Graph2D::new(WIDTH, HEIGHT, 50, 50, -10.0..10.0, -10.0..10.0);

        let pow = counter % 10.0;
        // println!("\n{:?}\n", pow);

        graph.add_cartesian(CoordinateStyle::default());
        graph.add_function(
            Box::new(move |x| x.powf(pow)),
            FunctionStyle::default().color(RED).thickness(2),
        );
        graph.add_function(Box::new(|x| x.sin()), FunctionStyle::default().color(BLUE).thickness(2));

        counter += 0.005;

        canvas.draw(&graph);

        window
            .update_with_buffer(&canvas.buffer_u32(), WIDTH, HEIGHT)
            .unwrap();
    }
}
