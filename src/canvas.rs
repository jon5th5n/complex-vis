use crate::color::{RGB, RGBA};

/// Trait for drawing anything arbitrary onto a `Canvas`.
pub trait Draw {
    /// Draws onto a `Canvas`.
    fn draw(&self, canvas: &mut Canvas);
}

#[derive(Debug, Clone)]
/// A `Canvas` is just a glorified pixel buffer with some usefull functionality.
pub struct Canvas {
    width: usize,
    height: usize,

    buffer: Vec<RGB>,
}

impl Canvas {
    /// Creates a new black canvas.
    pub fn new(width: usize, height: usize) -> Self {
        Canvas {
            width,
            height,
            buffer: vec![RGB { r: 0, g: 0, b: 0 }; width * height],
        }
    }
}

impl Canvas {
    /// Returns the width of the canvas.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the canvas.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns a reference to the pixel buffer of the canvas.
    pub fn buffer(&self) -> &Vec<RGB> {
        &self.buffer
    }

    /// Returns a mutabel reference to the pixel buffer of the canvas.
    pub fn buffer_mut(&mut self) -> &mut Vec<RGB> {
        &mut self.buffer
    }

    /// Returns the pixel buffer as a 32-bit buffer in the format `0RGB`.
    pub fn buffer_u32(&self) -> Vec<u32> {
        self.buffer
            .iter()
            .map(|c| (c.r as u32) << 16 | (c.g as u32) << 8 | (c.b as u32))
            .collect::<Vec<u32>>()
    }

    /// Checks if the pixel specified lays inside of the canvas.
    pub fn pixel_inside(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }

    /// Returns the color of the pixel at the specified position.
    pub fn get(&self, x: usize, y: usize) -> Option<&RGB> {
        self.buffer.get(y * self.width + x)
    }

    /// Sets the color of the pixel at the specified position.
    pub fn set(&mut self, x: usize, y: usize, color: RGB) -> Option<()> {
        *self.buffer.get_mut(y * self.width + x)? = color;
        Some(())
    }

    /// Fills the whole canvas with a given color.
    pub fn fill(&mut self, color: RGB) {
        self.buffer = vec![color; self.width * self.height];
    }
}

impl Canvas {
    /// Draws anything arbitrary implementing the `Draw` trait onto the canvas.
    pub fn draw<T>(&mut self, drawable: &T)
    where
        T: Draw,
    {
        drawable.draw(self);
    }

    /// Draws a single pixel onto the canvas.
    pub fn draw_pixel(&mut self, x: isize, y: isize, color: RGBA) -> Option<()> {
        if !self.pixel_inside(x, y) {
            return None;
        };

        let old_color = self.get(x as usize, y as usize)?;
        let new_color = old_color.add_rgba(color);
        self.set(x as usize, y as usize, new_color)
    }

    /// Draws a line onto the canvas.
    pub fn draw_line(&mut self, x1: isize, y1: isize, x2: isize, y2: isize, color: RGBA) {
        if x1 == x2 {
            let (start_y, end_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            for i in 0..(end_y - start_y) {
                self.draw_pixel(x1, start_y + i, color);
            }
        }

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();

        let abs_m = dy as f32 / dx as f32;
        match abs_m <= 1.0 {
            true => {
                let (start_x, start_y, end_x, end_y) = if x1 < x2 {
                    (x1, y1, x2, y2)
                } else {
                    (x2, y2, x1, y1)
                };

                let step = if start_y < end_y { 1 } else { -1 };

                let a = 2 * dy;
                let b = a - 2 * dx;
                let mut p = a - dx;
                self.draw_pixel(start_x, start_y, color);

                let mut offset = 0isize;
                for i in 1..=(end_x - start_x) {
                    match p < 0 {
                        true => {
                            p += a;
                        }
                        false => {
                            offset += step;
                            p += b;
                        }
                    }

                    self.draw_pixel(start_x + i, start_y + offset, color);
                }
            }
            false => {
                let (start_x, start_y, end_x, end_y) = if y1 < y2 {
                    (x1, y1, x2, y2)
                } else {
                    (x2, y2, x1, y1)
                };

                let step = if start_x < end_x { 1 } else { -1 };

                let a = 2 * dx;
                let b = a - 2 * dy;
                let mut p = a - dy;

                self.draw_pixel(start_x, start_y, color);

                let mut offset = 0isize;
                for i in 1..=(end_y - start_y) {
                    match p < 0 {
                        true => {
                            p += a;
                        }
                        false => {
                            offset += step;
                            p += b;
                        }
                    }

                    self.draw_pixel(start_x + offset, start_y + i, color);
                }
            }
        }
    }

    /// Draws a circle onto the canvas.
    pub fn draw_circle(&mut self, x: isize, y: isize, r: usize, color: RGBA) {
        if r == 0 {
            return;
        }

        let mut e = -(r as isize);
        let mut x_offset = r as isize;
        let mut y_offset = 0isize;

        while y_offset <= x_offset {
            self.draw_pixel(x + x_offset, y + y_offset, color);
            self.draw_pixel(x + x_offset, y - y_offset, color);
            self.draw_pixel(x - x_offset, y + y_offset, color);
            self.draw_pixel(x - x_offset, y - y_offset, color);

            self.draw_pixel(x + y_offset, y + x_offset, color);
            self.draw_pixel(x + y_offset, y - x_offset, color);
            self.draw_pixel(x - y_offset, y - x_offset, color);
            self.draw_pixel(x - y_offset, y + x_offset, color);

            e += 2 * y_offset + 1;
            y_offset += 1;
            if e >= 0 {
                e -= 2 * x_offset - 1;
                x_offset -= 1;
            }
        }
    }

    /// Draws a solid circle onto the canvas.
    pub fn draw_circle_solid(&mut self, x: isize, y: isize, r: usize, color: RGBA) {
        if r == 0 {
            return;
        }

        let mut e = -(r as isize);
        let mut x_offset = r as isize;
        let mut y_offset = 0isize;

        let dy = 2 * r;

        let mut left_buff = vec![0isize; dy + 1];
        let mut right_buff = vec![0isize; dy + 1];

        while y_offset <= x_offset {
            right_buff[(y + y_offset - (y - r as isize)) as usize] = x + x_offset;
            right_buff[(y - y_offset - (y - r as isize)) as usize] = x + x_offset;
            left_buff[(y + y_offset - (y - r as isize)) as usize] = x - x_offset;
            left_buff[(y - y_offset - (y - r as isize)) as usize] = x - x_offset;

            right_buff[(y + x_offset - (y - r as isize)) as usize] = x + y_offset;
            right_buff[(y - x_offset - (y - r as isize)) as usize] = x + y_offset;
            left_buff[(y + x_offset - (y - r as isize)) as usize] = x - y_offset;
            left_buff[(y - x_offset - (y - r as isize)) as usize] = x - y_offset;

            e += 2 * y_offset + 1;
            y_offset += 1;
            if e >= 0 {
                e -= 2 * x_offset - 1;
                x_offset -= 1;
            }
        }

        for i in 0..dy {
            let y = i as isize + (y - r as isize);
            let x1 = left_buff[i];
            let x2 = right_buff[i];

            for x in x1..x2 {
                self.draw_pixel(x, y, color);
            }
        }
    }

    /// Draws a polygon onto the canvas.
    pub fn draw_polygon(&mut self, vertices: Vec<(isize, isize)>, color: RGBA) {
        if vertices.is_empty() {
            return;
        }

        for i in 1..vertices.len() {
            let (x1, y1) = vertices[i];
            let (x2, y2) = vertices[i - 1];
            self.draw_line(x1, y1, x2, y2, color);
        }

        let (x1, y1) = vertices[0];
        let (x2, y2) = vertices[vertices.len() - 1];
        self.draw_line(x1, y1, x2, y2, color);
    }

    /// Draws a solid polygon onto the canvas.
    /// The vertices of the polygon have to be given in a clockwise order.
    pub fn draw_polygon_solid(
        &mut self,
        vertices: Vec<(isize, isize)>,
        clockwise: bool,
        color: RGBA,
    ) {
        if vertices.is_empty() {
            return;
        }

        let mut min_vert = 0;
        let mut max_vert = 0;
        for i in 0..vertices.len() {
            if vertices[i].1 < vertices[min_vert].1 {
                min_vert = i;
            }
            if vertices[i].1 > vertices[max_vert].1 {
                max_vert = i;
            }
        }

        let (start_x, start_y) = vertices[min_vert];

        let vertices = vertices
            .into_iter()
            .map(|(x, y)| (x - start_x, y - start_y))
            .collect::<Vec<_>>();

        let dy = (vertices[max_vert].1 + 1) as usize;

        let mut left_buff = vec![0isize; dy];
        let mut right_buff = vec![0isize; dy];

        let start_vert = if clockwise { min_vert } else { max_vert };
        let end_vert = if clockwise { max_vert } else { min_vert };

        let mut vert_index = start_vert;
        loop {
            let (x1, y1) = vertices[vert_index % vertices.len()];
            let (x2, y2) = vertices[(vert_index + 1) % vertices.len()];

            Self::polygon_buffer_line(&mut right_buff, true, x1, y1, x2, y2);

            vert_index += 1;
            if vert_index % vertices.len() == end_vert {
                break;
            }
        }

        let mut vert_index = end_vert;
        loop {
            let (x1, y1) = vertices[vert_index % vertices.len()];
            let (x2, y2) = vertices[(vert_index + 1) % vertices.len()];

            Self::polygon_buffer_line(&mut left_buff, false, x1, y1, x2, y2);

            vert_index += 1;
            if vert_index % vertices.len() == start_vert {
                break;
            }
        }

        for i in 0..dy {
            let y = i as isize + start_y;
            let x1 = left_buff[i] + start_x;
            let x2 = right_buff[i] + start_x;

            for x in x1..x2 {
                self.draw_pixel(x, y, color);
            }
        }
    }
}

impl Canvas {
    /// Computes a line for use of drawing solid polygons.
    fn polygon_buffer_line(
        buff: &mut Vec<isize>,
        right: bool,
        x1: isize,
        y1: isize,
        x2: isize,
        y2: isize,
    ) {
        if x1 == x2 {
            let (start_y, end_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            for i in 0..(end_y - start_y) {
                buff[i as usize] = x1;
            }
        }

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();

        let abs_m = dy as f32 / dx as f32;
        match abs_m <= 1.0 {
            true => {
                let (start_x, start_y, end_x, end_y) = if x1 < x2 {
                    (x1, y1, x2, y2)
                } else {
                    (x2, y2, x1, y1)
                };

                let step = if start_y < end_y { 1 } else { -1 };

                let a = 2 * dy;
                let b = a - 2 * dx;
                let mut p = a - dx;

                buff[start_y as usize] = start_x;
                let mut new_line = false;

                let mut offset = 0isize;
                for i in 1..=(end_x - start_x) {
                    match p < 0 {
                        true => {
                            p += a;
                        }
                        false => {
                            offset += step;
                            new_line = true;
                            p += b;
                        }
                    }

                    if right || new_line {
                        buff[(start_y + offset) as usize] = start_x + i;
                        new_line = false;
                    }
                }
            }
            false => {
                let (start_x, start_y, end_x, end_y) = if y1 < y2 {
                    (x1, y1, x2, y2)
                } else {
                    (x2, y2, x1, y1)
                };

                let step = if start_x < end_x { 1 } else { -1 };

                let a = 2 * dx;
                let b = a - 2 * dy;
                let mut p = a - dy;

                buff[start_y as usize] = start_x;

                let mut offset = 0isize;
                for i in 1..=(end_y - start_y) {
                    match p < 0 {
                        true => {
                            p += a;
                        }
                        false => {
                            offset += step;
                            p += b;
                        }
                    }

                    buff[(start_y + i) as usize] = start_x + offset;
                }
            }
        }
    }
}
