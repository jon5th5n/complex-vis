use std::time::Instant;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

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
    let mut var: f64 = 10.0;
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

        let (m_x, m_y) = window.get_mouse_pos(MouseMode::Pass).unwrap();
        let m_p = window.get_mouse_down(MouseButton::Left);

        match m_p {
            true => canvas.draw_polyline_aa(500.0, 500.0, m_x, m_y, var as f32, RED),
            false => canvas.draw_polyline(
                500,
                500,
                m_x.round() as isize,
                m_y.round() as isize,
                var.round() as u32,
                RED,
            ),
        }

        // let mut graph = Graph2D::new(WIDTH, HEIGHT, 50, 50, -10.0..10.0, -10.0..10.0);

        // // let pow = counter % 10.0;
        // // println!("\n{:?}\n", pow);

        // if window.is_key_down(Key::Up) {
        //     var += 0.01;
        // }
        // if window.is_key_down(Key::Down) {
        //     var -= 0.01;
        // }
        // println!("pow: {:?}", var);

        // graph.add_cartesian(CoordinateStyle::default());

        // graph.add_function(
        //     Box::new(move |x| {
        //         Complex::new_real(x)
        //             .pow(Complex::new_real(var))
        //             .unwrap()
        //             .re()
        //     }),
        //     FunctionStyle::default().color(BLUE).thickness(2.0),
        // );
        // graph.add_function(
        //     Box::new(move |x| {
        //         Complex::new_real(x)
        //             .pow(Complex::new_real(var))
        //             .unwrap()
        //             .im()
        //     }),
        //     FunctionStyle::default().color(BLUE).thickness(1.0),
        // );

        // graph.add_function(
        //     Box::new(move |x| (x).cos()),
        //     FunctionStyle::default().color(GREEN).thickness(1.0),
        // );
        // graph.add_function(
        //     Box::new(move |x| (x).sin()),
        //     FunctionStyle::default().color(GREEN).thickness(1.0),
        // );

        // graph.add_function(
        //     Box::new(move |x| {
        //         Complex::new_real(var)
        //             .pow(Complex::new_real(x / std::f64::consts::PI))
        //             .unwrap()
        //             .re()
        //     }),
        //     FunctionStyle::default().color(RED).thickness(2.0),
        // );
        // graph.add_function(
        //     Box::new(move |x| {
        //         Complex::new_real(var)
        //             .pow(Complex::new_real(x / std::f64::consts::PI))
        //             .unwrap()
        //             .im()
        //     }),
        //     FunctionStyle::default().color(RED).thickness(1.0),
        // );

        // graph.add_function(
        //     Box::new(move |x| Complex::e().pow(Complex::new_imaginary(x)).unwrap().re()),
        //     FunctionStyle::default().color(GREEN).thickness(2.0),
        // );
        // graph.add_function(
        //     Box::new(move |x| Complex::e().pow(Complex::new_imaginary(x)).unwrap().im()),
        //     FunctionStyle::default().color(GREEN).thickness(2.0),
        // );

        // counter += 0.005;

        // canvas.draw(&graph);

        window
            .update_with_buffer(&canvas.buffer_u32(), WIDTH, HEIGHT)
            .unwrap();
    }
}
