use minifb::{Key, Window, WindowOptions};

mod canvas;
use canvas::*;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;

fn main() {
    println!("hello world");

    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    let mut window = Window::new("Complex Stuff", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        canvas.fill(RGB { r: 0, g: 0, b: 0 });

        let x1 = 500isize;
        let y1 = 500isize;

        let (mouse_x, mouse_y) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();

        canvas.draw_line(
            x1,
            y1,
            mouse_x as isize,
            mouse_y as isize,
            RGBA {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        );

        canvas.draw_circle_solid(
            x1,
            y1,
            (((x1 - mouse_x as isize) * (x1 - mouse_x as isize)
                + (y1 - mouse_y as isize) * (y1 - mouse_y as isize)) as f32)
                .sqrt() as usize,
            RGBA {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        );

        canvas.draw_circle(
            x1,
            y1,
            (((x1 - mouse_x as isize) * (x1 - mouse_x as isize)
                + (y1 - mouse_y as isize) * (y1 - mouse_y as isize)) as f32)
                .sqrt() as usize,
            RGBA {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
        );

        canvas.draw_polygon_solid(
            vec![(100, 100), (220, 200), (400, 200), (200, 250)],
            true,
            RGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
        );

        // canvas.draw_polygon_solid(
        //     vec![(400, 400), (600, 400), (600, 500), (500, 900)],
        //     true,
        //     RGBA {
        //         r: 0,
        //         g: 255,
        //         b: 0,
        //         a: 255,
        //     },
        // );

        window
            .update_with_buffer(&canvas.buffer_u32(), WIDTH, HEIGHT)
            .unwrap();
    }
}
