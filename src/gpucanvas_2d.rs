use crate::{GPUView, GPUViewFrame, ShaderDescriptor, Vertex};
use std::{cell::RefCell, ops::Range, sync::Arc};

struct GPUCanvas2DShaderDescriptor {}

impl GPUCanvas2DShaderDescriptor {
    fn new() -> Self {
        Self {}
    }

    fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(self))
    }
}

impl ShaderDescriptor for GPUCanvas2DShaderDescriptor {
    fn initialize(&mut self, device: &wgpu::Device) {
        return;
    }

    fn update_buffers(&self, queue: &wgpu::Queue) {
        return;
    }

    fn shader_source(&self) -> wgpu::ShaderSource {
        wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
    }

    fn bind_group_and_layout(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shader Descripot Bind Group Layout"),
            entries: &[],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shader Descriptor Bind Group"),
            layout: &bind_group_layout,
            entries: &[],
        });

        (bind_group, bind_group_layout)
    }
}

pub struct GPUCanvas2D<'a> {
    x_margin: f32,
    y_margin: f32,

    x_range: Range<f32>,
    y_range: Range<f32>,

    shader_descriptor: Arc<RefCell<GPUCanvas2DShaderDescriptor>>,
    view: Arc<RefCell<GPUView<'a>>>,
}

impl<'a> GPUCanvas2D<'a> {
    pub fn new(view_frame: GPUViewFrame) -> Self {
        let shader_descriptor = GPUCanvas2DShaderDescriptor::new().into_arc_ref_cell();

        Self {
            x_margin: 0.05,
            y_margin: 0.05,
            x_range: -1.0..1.0,
            y_range: -1.0..1.0,
            shader_descriptor: shader_descriptor.clone(),
            view: GPUView::new(view_frame, shader_descriptor).into_arc_ref_cell(),
        }
    }

    pub fn get_view(&self) -> Arc<RefCell<GPUView<'a>>> {
        self.view.clone()
    }

    pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
        self.view.as_ref().borrow_mut().set_clear_color(clear_color);
    }

    pub fn add_function(&mut self, f: fn(f32) -> f32) {
        let mut vertices = Vec::new();

        let sample_freq = 100u32;

        let x_start = self.x_range.start;
        let x_len = self.x_range.end - x_start;

        let step = x_len / sample_freq as f32;

        let mut prev_x = x_start;
        let mut prev_y = f(prev_x);
        for i in 1..=sample_freq {
            let x = x_start + (step * i as f32);
            let y = f(x);

            vertices_add_line(
                &mut vertices,
                [prev_x, prev_y],
                [x, y],
                0.01,
                [0.0, 0.0, 0.0, 1.0],
            );

            prev_x = x;
            prev_y = y;
        }

        self.view
            .as_ref()
            .borrow_mut()
            .append_render_vertices(&mut vertices);
    }
}

fn vertices_add_line(
    vertices: &mut Vec<Vertex>,
    end1: [f32; 2],
    end2: [f32; 2],
    width: f32,
    color: [f32; 4],
) {
    let normal = [end2[1] - end1[1], -(end2[0] - end1[0])];
    let normal_len = (normal[0] * normal[0] + normal[1] * normal[1]).sqrt();
    let normal_norm = [normal[0] / normal_len, normal[1] / normal_len];
    let normal_width = [normal_norm[0] * width, normal_norm[1] * width];

    let corner11 = [
        end1[0] + normal_width[0] / 2.0,
        end1[1] + normal_width[1] / 2.0,
    ];
    let corner12 = [
        end1[0] - normal_width[0] / 2.0,
        end1[1] - normal_width[1] / 2.0,
    ];
    let corner21 = [
        end2[0] + normal_width[0] / 2.0,
        end2[1] + normal_width[1] / 2.0,
    ];
    let corner22 = [
        end2[0] - normal_width[0] / 2.0,
        end2[1] - normal_width[1] / 2.0,
    ];

    vertices.append(&mut vec![
        Vertex {
            position: [corner11[0], corner11[1], 0.0],
            color,
        },
        Vertex {
            position: [corner12[0], corner12[1], 0.0],
            color,
        },
        Vertex {
            position: [corner21[0], corner21[1], 0.0],
            color,
        },
        Vertex {
            position: [corner12[0], corner12[1], 0.0],
            color,
        },
        Vertex {
            position: [corner21[0], corner21[1], 0.0],
            color,
        },
        Vertex {
            position: [corner22[0], corner22[1], 0.0],
            color,
        },
    ]);
}
