use std::vec;
use std::{fmt::Debug, time::Instant};

use minifb::{Key, Window, WindowOptions};

use complex_stuff::*;

use drawing_stuff::canvas::{Canvas, Draw};
use drawing_stuff::color::*;
use drawing_stuff::drawables::*;

// mod sdf;
// use sdf::*;

mod graphing;
use graphing::*;

mod graph2d;
use graph2d::*;

// mod graph2dsdf;
// use graph2dsdf::*;

mod vector;
use vector::Vector3;

mod quaternion;
use quaternion::Quaternion;

mod tmp;
use tmp::*;

mod gpu;
use gpu::*;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;

fn main() {
    unsafe { backtrace_on_stack_overflow::enable() };

    println!("hello world");

    let mut sdfc = pollster::block_on(SDF2Constructor::new(1000, 1000));

    let sdf = Circle2D {
        center: (500.0, 500.0),
        radius: 50.0,
        color: 0x00000000,
    };

    let bsdf = Box::new(sdf);

    println!("A");

    for _ in 0..1000 {
        sdfc.add_sdf(bsdf.clone());
    }

    println!("B");

    sdfc.compile();

    println!("C");

    let mut last_tick = Instant::now();

    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    let mut window = Window::new("Complex Stuff", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta = now - last_tick;
        println!("{}", delta.as_millis());
        last_tick = now;

        let res = pollster::block_on(sdfc.run());

        window.update_with_buffer(&res, WIDTH, HEIGHT).unwrap();
    }
}
