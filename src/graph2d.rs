use std::{ops::Range, sync::Arc};

use crate::gpu_canvas::*;
use drawing_stuff::{canvas::Canvas as CPUCanvas, rgba::*};

pub struct Graph2D {
    width: u32,
    height: u32,

    gpu_canvas: GPUCanvas,
    cpu_canvas: CPUCanvas<RGBA>,

    x_margin: u32,
    y_margin: u32,

    x_range: Range<f32>,
    y_range: Range<f32>,
}

impl Graph2D {
    pub fn new(
        gpu_device: Arc<wgpu::Device>,
        gpu_queue: Arc<wgpu::Queue>,

        width: u32,
        height: u32,

        x_margin: u32,
        y_margin: u32,

        x_range: Range<f32>,
        y_range: Range<f32>,
    ) -> Self {
        let gpu_canvas = GPUCanvas::new(width, height, gpu_device, gpu_queue);
        let cpu_canvas = CPUCanvas::new(width as usize, height as usize);

        Self {
            width,
            height,
            gpu_canvas,
            cpu_canvas,
            x_margin,
            y_margin,
            x_range,
            y_range,
        }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;

        self.gpu_canvas.resize(self.width, self.height);
        self.cpu_canvas = CPUCanvas::new(self.width as usize, self.height as usize);
    }

    pub fn build(&self) -> &wgpu::Texture {
        self.gpu_canvas.texture()
    }
}
