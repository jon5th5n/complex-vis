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

    pub fn scale_range(&mut self, scale: f32) {
        let x_diff = self.x_range_len() * (scale - 1.0);
        let y_diff = self.y_range_len() * (scale - 1.0);

        self.x_range = (self.x_range.start - (x_diff * 0.5))..(self.x_range.end + (x_diff * 0.5));
        self.y_range = (self.y_range.start - (y_diff * 0.5))..(self.y_range.end + (y_diff * 0.5));
    }

    pub fn offset_range(&mut self, offset: (f32, f32)) {
        self.x_range = (self.x_range.start + offset.0)..(self.x_range.end + offset.0);
        self.y_range = (self.y_range.start + offset.1)..(self.y_range.end + offset.1);
    }

    fn x_range_len(&self) -> f32 {
        self.x_range.end - self.x_range.start
    }

    fn y_range_len(&self) -> f32 {
        self.y_range.end - self.y_range.start
    }

    fn global_to_screen(&self, global: (f32, f32)) -> (f32, f32) {
        let (gx, gy) = global;

        let lx = (2.0 * gx - self.x_range.start - self.x_range.end) / self.x_range_len();
        let ly = (2.0 * gy - self.y_range.start - self.y_range.end) / self.y_range_len();

        (lx, ly)
    }

    pub fn add_function(&mut self, f: fn(f32) -> f32) {
        let mut points = Vec::new();

        let sample_freq = 10000u32;

        let x_start = self.x_range.start;
        let x_len = self.x_range.end - x_start;

        let step = x_len / sample_freq as f32;

        for i in 0..=sample_freq {
            let gx = x_start + (step * i as f32);
            let gy = f(gx);

            let (lx, ly) = self.global_to_screen((gx, gy));

            points.push((lx, ly));
        }

        let mut vertices = Vec::new();

        vertices_add_polyline(&mut vertices, points, 0.01, [1.0, 0.0, 0.0, 1.0]);

        self.view
            .as_ref()
            .borrow_mut()
            .append_render_vertices(&mut vertices);
    }
}

#[derive(Debug)]
struct LineFunction {
    point: (f32, f32),
    slope: f32,
}

impl LineFunction {
    fn evaluate_at(&self, x: f32) -> f32 {
        self.slope * (x - self.point.0) + self.point.1
    }

    fn intersection(&self, other: &Self) -> (f32, f32) {
        let x = (self.slope * self.point.0 - self.point.1 - other.slope * other.point.0
            + other.point.1)
            / (self.slope - other.slope);
        let y = self.slope * (x - self.point.0) + self.point.1;

        (x, y)
    }
}

fn vertices_add_polyline(
    vertices: &mut Vec<Vertex>,
    points: Vec<(f32, f32)>,
    width: f32,
    color: [f32; 4],
) {
    let width_half = width / 2.0;

    let mut top_points: Vec<Option<(f32, f32)>> = vec![None; points.len()];
    let mut bot_points: Vec<Option<(f32, f32)>> = vec![None; points.len()];

    let mut prev_top_line: Option<LineFunction> = None;
    let mut prev_bot_line: Option<LineFunction> = None;
    for i in 1..points.len() {
        let point = points[i];
        let prev_point = points[i - 1];

        let dx = point.0 - prev_point.0;
        let dy = point.1 - prev_point.1;

        let slope = dy / dx;

        if !dx.is_finite() || !dy.is_finite() || !slope.is_finite() {
            continue;
        }

        let dlen = (dx * dx + dy * dy).sqrt();

        let dx_norm = dx.signum() * (-dy) / dlen;
        let dy_norm = dx.signum() * (dx) / dlen;

        if i == (points.len() - 1) {
            let e_top = (
                point.0 + width_half * dx_norm,
                point.1 + width_half * dy_norm,
            );
            let e_bot = (
                point.0 - width_half * dx_norm,
                point.1 - width_half * dy_norm,
            );

            top_points[i] = Some(e_top);
            bot_points[i] = Some(e_bot);
        }

        let p_top = (
            prev_point.0 + width_half * dx_norm,
            prev_point.1 + width_half * dy_norm,
        );
        let p_bot = (
            prev_point.0 - width_half * dx_norm,
            prev_point.1 - width_half * dy_norm,
        );

        let top_line = LineFunction {
            point: p_top,
            slope,
        };
        let bot_line = LineFunction {
            point: p_bot,
            slope,
        };

        if prev_top_line.is_none()
            || prev_bot_line.is_none()
            || ((top_line.slope - prev_top_line.as_ref().unwrap().slope).abs() <= 0.001)
            || ((bot_line.slope - prev_bot_line.as_ref().unwrap().slope).abs() <= 0.001)
        // || true
        {
            top_points[i - 1] = Some(p_top);
            bot_points[i - 1] = Some(p_bot);

            prev_top_line = Some(top_line);
            prev_bot_line = Some(bot_line);

            continue;
        }

        let top_intersect = top_line.intersection(&prev_top_line.as_ref().unwrap());
        let bot_intersect = bot_line.intersection(&prev_bot_line.unwrap());

        if !top_intersect.0.is_finite()
            || !top_intersect.1.is_finite()
            || !bot_intersect.0.is_finite()
            || !bot_intersect.1.is_finite()
            || !top_line.slope.is_finite()
            || !top_line.point.0.is_finite()
            || !top_line.point.1.is_finite()
            || !bot_line.slope.is_finite()
            || !bot_line.point.0.is_finite()
            || !bot_line.point.1.is_finite()
            || top_intersect.1 < 0.0 && top_intersect.0.abs() > 0.1
            || bot_intersect.1 < 0.0 && bot_intersect.0.abs() > 0.1
        // || dx_norm.abs() == 1.0
        {
            println!("{:?}", top_intersect);
            println!("{:?}", bot_intersect);
            println!("{:?}, {:?}, {:?}, {:?}", p_top, p_bot, dx_norm, dy_norm);
            println!(
                "{:?}, ({:?} / {:?}), ({:?} - {:?})",
                slope, dy, dx, point.1, prev_point.1
            );
            println!();
        }

        top_points[i - 1] = Some(top_intersect);
        bot_points[i - 1] = Some(bot_intersect);

        prev_top_line = Some(top_line);
        prev_bot_line = Some(bot_line);
    }

    let mut counter = 0;

    let mut a = None;
    let mut b = None;
    for i in 0..points.len() {
        if a.is_none() || b.is_none() {
            a = top_points[i];
            b = bot_points[i];
            continue;
        }

        let c = bot_points[i];
        let d = top_points[i];

        if c.is_none() || d.is_none() {
            continue;
        }

        vertices.append(&mut vec![
            Vertex {
                position: [a.unwrap().0, a.unwrap().1, 0.0],
                color,
            },
            Vertex {
                position: [b.unwrap().0, b.unwrap().1, 0.0],
                color,
            },
            Vertex {
                position: [c.unwrap().0, c.unwrap().1, 0.0],
                color,
            },
            Vertex {
                position: [a.unwrap().0, a.unwrap().1, 0.0],
                color,
            },
            Vertex {
                position: [c.unwrap().0, c.unwrap().1, 0.0],
                color,
            },
            Vertex {
                position: [d.unwrap().0, d.unwrap().1, 0.0],
                color,
            },
        ]);

        a = d;
        b = c;
    }

    // println!(
    //     "{:?}",
    //     [top_points[0], top_points[1], top_points[2], top_points[3]]
    // );
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
