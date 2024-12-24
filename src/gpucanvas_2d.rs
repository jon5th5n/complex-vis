use crate::color::*;
use crate::decimal_math::*;
use crate::graph::*;
use crate::math::lerp;
use crate::TextSection;
use crate::{GPUView, GPUViewFrame, ShaderDescriptor, Vertex};

use fraction::ToPrimitive;
use wgpu::util::DeviceExt;
use wgpu_text::glyph_brush::HorizontalAlign;
use wgpu_text::glyph_brush::Layout;
use wgpu_text::glyph_brush::SectionBuilder;
use wgpu_text::glyph_brush::Text;
use wgpu_text::glyph_brush::VerticalAlign;

use std::{cell::RefCell, ops::Range, sync::Arc};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::NoUninit)]
struct GPUCanvas2DShaderEnv {
    range_start: [f32; 2],
    range_end: [f32; 2],
}

struct GPUCanvas2DShaderDescriptor {
    enviroment: GPUCanvas2DShaderEnv,

    enviroment_buffer: Option<wgpu::Buffer>,

    is_initialized: bool,
    enviroment_changed: bool,
}

impl GPUCanvas2DShaderDescriptor {
    fn new(enviroment: GPUCanvas2DShaderEnv) -> Self {
        Self {
            enviroment,
            enviroment_buffer: None,
            is_initialized: false,
            enviroment_changed: false,
        }
    }

    fn enviroment_get_mut(&mut self) -> &mut GPUCanvas2DShaderEnv {
        self.enviroment_changed = true;
        &mut self.enviroment
    }

    fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(self))
    }
}

impl ShaderDescriptor for GPUCanvas2DShaderDescriptor {
    fn initialize(&mut self, device: &wgpu::Device) -> anyhow::Result<()> {
        self.enviroment_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("GPUCanvas2DShaderDescriptor Enviroment Variable Buffer"),
                contents: bytemuck::bytes_of(&self.enviroment),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            },
        ));

        self.is_initialized = true;

        Ok(())
    }

    fn update_buffers(&mut self, queue: &wgpu::Queue) -> anyhow::Result<()> {
        if !self.is_initialized {
            return Err(anyhow::Error::msg(
                "Cannot update buffer of uninitialized shader descriptor.",
            ));
        }

        if self.enviroment_changed {
            let new_data = bytemuck::bytes_of(&self.enviroment);

            let buffer = self.enviroment_buffer.as_ref().unwrap();
            queue.write_buffer(buffer, 0, new_data);

            self.enviroment_changed = false;
        }

        Ok(())
    }

    fn shader_source(&self) -> wgpu::ShaderSource {
        wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
    }

    fn bind_group_and_layout(
        &self,
        device: &wgpu::Device,
    ) -> anyhow::Result<(wgpu::BindGroup, wgpu::BindGroupLayout)> {
        if !self.is_initialized {
            return Err(anyhow::Error::msg(
                "Cannot get BindGroup and BindGroupLayout of uninitialized GPUCanvas2DShaderDescriptor.",
            ));
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shader Descripot Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shader Descriptor Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    self.enviroment_buffer
                        .as_ref()
                        .unwrap()
                        .as_entire_buffer_binding(),
                ),
            }],
        });

        Ok((bind_group, bind_group_layout))
    }
}

pub struct GPUCanvas2D<P>
where
    P: Default,
{
    style: EnviromentStyle,

    x_range: Range<f64>, // coordinate space
    y_range: Range<f64>, // coordinate space

    functions: Vec<FunctionGraph<f64, P, f64>>,
    parameter: P,

    shader_descriptor: Arc<RefCell<GPUCanvas2DShaderDescriptor>>,
    view: Arc<RefCell<GPUView>>,

    style_changed: bool,
    range_changed: bool,
    function_changed: bool,
}

impl<P> GPUCanvas2D<P>
where
    P: Default,
{
    pub fn new(view_frame: GPUViewFrame) -> Self {
        let shader_enviroment = GPUCanvas2DShaderEnv {
            range_start: [-1.0, -1.0],
            range_end: [1.0, 1.0],
        };
        let shader_descriptor =
            GPUCanvas2DShaderDescriptor::new(shader_enviroment).into_arc_ref_cell();

        Self {
            style: EnviromentStyle::default(),
            x_range: -1.0..1.0,
            y_range: -1.0..1.0,
            functions: Vec::new(),
            parameter: P::default(),
            shader_descriptor: shader_descriptor.clone(),
            view: GPUView::new(view_frame, shader_descriptor).into_arc_ref_cell(),
            style_changed: true,
            range_changed: true,
            function_changed: true,
        }
    }

    pub fn set_style(&mut self, style: EnviromentStyle) {
        self.style = style;
        self.style_changed = true;
    }

    pub fn style_get_mut(&mut self) -> &mut EnviromentStyle {
        self.style_changed = true;
        &mut self.style
    }

    pub fn parameter_get_mut(&mut self) -> &mut P {
        self.function_changed = true;
        &mut self.parameter
    }

    fn update_shader_env_range(&mut self) {
        let mut tmp = self.shader_descriptor.borrow_mut();
        let env = tmp.enviroment_get_mut();

        env.range_start = [self.x_range.start as f32, self.y_range.start as f32];
        env.range_end = [self.x_range.end as f32, self.y_range.end as f32];
    }

    pub fn get_view(&self) -> Arc<RefCell<GPUView>> {
        self.view.clone()
    }

    pub fn set_clear_color(&mut self, clear_color: RGBA) {
        self.view
            .as_ref()
            .borrow_mut()
            .set_clear_color(clear_color.into());
    }

    pub fn x_range(&self) -> &Range<f64> {
        &self.x_range
    }

    pub fn y_range(&self) -> &Range<f64> {
        &self.y_range
    }

    pub fn scale_range(&mut self, scale: (f64, f64)) {
        static MAX_RANGE: f32 = 5.0;
        static MIN_RANGE: f32 = 0.2;

        let x_diff = self.x_range_len() * (scale.0 - 1.0);
        let y_diff = self.y_range_len() * (scale.1 - 1.0);

        let new_x_range =
            (self.x_range.start - (x_diff * 0.5))..(self.x_range.end + (x_diff * 0.5));
        let new_y_range =
            (self.y_range.start - (y_diff * 0.5))..(self.y_range.end + (y_diff * 0.5));

        if !new_x_range.start.is_finite()
            || !new_x_range.start.is_finite()
            || !new_x_range.end.is_finite()
            || !new_x_range.end.is_finite()
            || !new_y_range.start.is_finite()
            || !new_y_range.start.is_finite()
            || !new_y_range.end.is_finite()
            || !new_y_range.end.is_finite()
        {
            return;
        }

        self.x_range = new_x_range;
        self.y_range = new_y_range;

        self.range_changed = true;
        self.update_shader_env_range();
    }

    pub fn offset_range(&mut self, offset: (f64, f64)) {
        self.x_range = (self.x_range.start + offset.0)..(self.x_range.end + offset.0);
        self.y_range = (self.y_range.start + offset.1)..(self.y_range.end + offset.1);

        self.range_changed = true;
        self.update_shader_env_range();
    }

    pub fn x_range_len(&self) -> f64 {
        self.x_range.end - self.x_range.start
    }

    pub fn y_range_len(&self) -> f64 {
        self.y_range.end - self.y_range.start
    }

    /// Coordinate transform from range to `-1..1`
    fn global_to_screen(&self, global: (f64, f64)) -> (f32, f32) {
        let (gx, gy) = global;

        let lx = (2.0 * gx - self.x_range.start - self.x_range.end) / self.x_range_len();
        let ly = (2.0 * gy - self.y_range.start - self.y_range.end) / self.y_range_len();

        (lx as f32, ly as f32)
    }

    fn calculate_dynamic_spacing(range_len: f64, num_steps: u32) -> Decimal {
        let range_len = decimal_from_to_string(range_len);
        let num_steps = Decimal::from(num_steps);

        let base = range_len / num_steps;

        let steps = [Decimal::from(1), Decimal::from(2), Decimal::from(5)].into_iter();

        let closest = steps
            .map(|step| {
                let log = decimal_log10_ceil(&(base.clone() / step.clone()));
                let exp = decimal_exp10(log);
                step * exp
            })
            .map(|exp| (exp.clone(), (exp - base.clone()).abs()))
            .min_by(|x, y| x.1.cmp(&y.1))
            .unwrap();

        closest.0
    }

    pub fn add_function_graph(&mut self, function_graph: FunctionGraph<f64, P, f64>) {
        self.functions.push(function_graph);
        self.function_changed = true;
    }

    fn screen_constant(&self, value: f64) -> f32 {
        (value * ((self.x_range_len() + self.y_range_len()) / 2.0)) as f32
    }

    fn display_refresh_required(&self) -> bool {
        self.style_changed || self.range_changed || self.function_changed
    }

    fn display_reset_refresh(&mut self) {
        self.style_changed = false;
        self.range_changed = false;
        self.function_changed = false;
    }

    pub fn display_clear(&mut self) {
        let mut view = self.view.as_ref().borrow_mut();

        view.clear_render_vertices();
        view.clear_text_sections();
    }

    pub fn display(&mut self) {
        if !self.display_refresh_required() {
            return;
        }

        self.display_reset_refresh();

        self.display_clear();

        self.display_enviroment();
        self.display_function_graphs();
    }

    fn display_enviroment(&mut self) {
        //-- screen mapping of global zero

        let (sx0, sy0) = self.global_to_screen((0.0, 0.0));

        //-- ranges

        let x_range_start = self.x_range.start;
        let x_range_end = self.x_range.end;

        let y_range_start = self.y_range.start;
        let y_range_end = self.y_range.end;

        //-- spacings in decimal representation

        let (x_step_spacing, x_substeps) = match &self.style.x.spacing {
            GridSpacing::Dynamic { steps, substeps } => (
                Self::calculate_dynamic_spacing(self.x_range_len(), *steps),
                *substeps,
            ),
            GridSpacing::Fixed { spacing, substeps } => (spacing.clone(), *substeps),
        };

        let (y_step_spacing, y_substeps) = match &self.style.y.spacing {
            GridSpacing::Dynamic { steps, substeps } => (
                Self::calculate_dynamic_spacing(self.y_range_len(), *steps),
                *substeps,
            ),
            GridSpacing::Fixed { spacing, substeps } => (spacing.clone(), *substeps),
        };

        let x_substep_spacing = &x_step_spacing / (x_substeps + 1) as f64;
        let y_substep_spacing = &y_step_spacing / (y_substeps + 1) as f64;

        //-- floating point representations of the spacings

        let x_step_spacing_f64 = x_step_spacing.to_f64().expect(Self::ERROR_DEC_TO_F64);
        let y_step_spacing_f64 = y_step_spacing.to_f64().expect(Self::ERROR_DEC_TO_F64);
        let x_substep_spacing_f64 = x_substep_spacing.to_f64().expect(Self::ERROR_DEC_TO_F64);
        let y_substep_spacing_f64 = y_substep_spacing.to_f64().expect(Self::ERROR_DEC_TO_F64);

        //-- offset to make the ranges symetric to zero

        let x_sym_offset = x_range_start + (self.x_range_len() / 2.0);
        let x_sym_offset = (x_sym_offset / x_step_spacing_f64).round() * x_step_spacing_f64;

        let y_sym_offset = y_range_start + (self.y_range_len() / 2.0);
        let y_sym_offset = (y_sym_offset / y_step_spacing_f64).round() * y_step_spacing_f64;

        //-- x spacing indices

        let x_substep_start_index =
            ((x_range_start - x_sym_offset) / x_substep_spacing_f64).ceil() as i32;
        let x_substep_end_index =
            ((x_range_end - x_sym_offset) / x_substep_spacing_f64).floor() as i32;

        let x_step_start_index =
            ((x_range_start - x_sym_offset) / x_step_spacing_f64).ceil() as i32;
        let x_step_end_index = ((x_range_end - x_sym_offset) / x_step_spacing_f64).floor() as i32;

        let x_substep_range = x_substep_start_index..=x_substep_end_index;
        let x_step_range = x_step_start_index..=x_step_end_index;

        //-- y spacing indices

        let y_substep_start_index =
            ((y_range_start - y_sym_offset) / y_substep_spacing_f64).ceil() as i32;
        let y_substep_end_index =
            ((y_range_end - y_sym_offset) / y_substep_spacing_f64).floor() as i32;

        let y_step_start_index =
            ((y_range_start - y_sym_offset) / y_step_spacing_f64).ceil() as i32;
        let y_step_end_index = ((y_range_end - y_sym_offset) / y_step_spacing_f64).floor() as i32;

        let y_substep_range = y_substep_start_index..=y_substep_end_index;
        let y_step_range = y_step_start_index..=y_step_end_index;

        // println!(
        //     "x-range: {{{:?}, len: {:?}}}",
        //     self.x_range,
        //     self.x_range_len()
        // );

        // println!("x-sym-offset: {:?}", x_sym_offset);

        // println!(
        //     "x-step: {:?}, x-substep: {:?}",
        //     x_step_spacing_f64, x_substep_spacing_f64
        // );

        // println!("x-substep-range {:?}", x_substep_range);

        //-- grid ---

        if let Some(subgrid_style) = self.style.x.subgrid {
            for i in x_substep_range.clone() {
                let x = (i as f64 * x_substep_spacing_f64) + x_sym_offset;
                let (sx, _) = self.global_to_screen((x, 0.0));

                self.vertices_add_line(
                    [sx, -1.0],
                    [sx, 1.0],
                    subgrid_style.thickness,
                    subgrid_style.color,
                );
            }
        }

        if let Some(subgrid_style) = self.style.y.subgrid {
            for i in y_substep_range.clone() {
                let y = (i as f64 * y_substep_spacing_f64) + y_sym_offset;
                let (_, sy) = self.global_to_screen((0.0, y));

                self.vertices_add_line(
                    [-1.0, sy],
                    [1.0, sy],
                    subgrid_style.thickness,
                    subgrid_style.color,
                );
            }
        }

        if let Some(grid_style) = self.style.x.grid {
            for i in x_step_range.clone() {
                let x = (i as f64 * x_step_spacing_f64) + x_sym_offset;
                let (sx, _) = self.global_to_screen((x, 0.0));

                self.vertices_add_line(
                    [sx, -1.0],
                    [sx, 1.0],
                    grid_style.thickness,
                    grid_style.color,
                );
            }
        }

        if let Some(grid_style) = self.style.y.grid {
            for i in y_step_range.clone() {
                let y = (i as f64 * y_step_spacing_f64) + y_sym_offset;
                let (_, sy) = self.global_to_screen((0.0, y));

                self.vertices_add_line(
                    [-1.0, sy],
                    [1.0, sy],
                    grid_style.thickness,
                    grid_style.color,
                );
            }
        }

        //-----------

        //-- axes ---

        if let Some(axis_style) = self.style.x.axis {
            self.vertices_add_line(
                [-1.0, sy0],
                [1.0, sy0],
                axis_style.thickness,
                axis_style.color,
            );
        }

        if let Some(axis_style) = self.style.y.axis {
            self.vertices_add_line(
                [sx0, -1.0],
                [sx0, 1.0],
                axis_style.thickness,
                axis_style.color,
            );
        }

        //-----------

        //-- ticks --

        if let Some(subtick_style) = self.style.x.subtick {
            for i in x_substep_range.clone() {
                let x = (i as f64 * x_substep_spacing_f64) + x_sym_offset;

                let (sx, sy) = self.global_to_screen((x, 0.0));

                self.vertices_add_polyline(
                    &[
                        [sx, sy + subtick_style.length / 2.0],
                        [sx, sy - subtick_style.length / 2.0],
                    ],
                    self.screen_constant(subtick_style.thickness as f64),
                    subtick_style.color,
                );
            }
        }

        if let Some(subtick_style) = self.style.y.subtick {
            for i in y_substep_range.clone() {
                let y = (i as f64 * y_substep_spacing_f64) + y_sym_offset;
                let (sx, sy) = self.global_to_screen((0.0, y));

                self.vertices_add_polyline(
                    &[
                        [sx + subtick_style.length / 2.0, sy],
                        [sx - subtick_style.length / 2.0, sy],
                    ],
                    self.screen_constant(subtick_style.thickness as f64),
                    subtick_style.color,
                );
            }
        }

        if let Some(tick_style) = self.style.x.tick {
            for i in x_step_range.clone() {
                let x = (i as f64 * x_step_spacing_f64) + x_sym_offset;
                let (sx, sy) = self.global_to_screen((x, 0.0));

                self.vertices_add_polyline(
                    &[
                        [sx, sy + tick_style.length / 2.0],
                        [sx, sy - tick_style.length / 2.0],
                    ],
                    self.screen_constant(tick_style.thickness as f64),
                    tick_style.color,
                );
            }
        }

        if let Some(tick_style) = self.style.y.tick {
            for i in y_step_range.clone() {
                let y = (i as f64 * y_step_spacing_f64) + y_sym_offset;
                let (sx, sy) = self.global_to_screen((0.0, y));

                self.vertices_add_polyline(
                    &[
                        [sx + tick_style.length / 2.0, sy],
                        [sx - tick_style.length / 2.0, sy],
                    ],
                    tick_style.thickness,
                    tick_style.color,
                );
            }
        }

        //-----------

        //-- text --

        if let Some(text_style) = &self.style.text {
            let text_size = text_style.size;
            let text_font = &text_style.font;
            let text_max_digits = text_style.max_digits;

            {
                let start_index = (x_range_start / x_step_spacing_f64).ceil() as i128;
                let end_index = (x_range_end / x_step_spacing_f64).floor() as i128;

                for i in start_index..=end_index {
                    if i == 0 {
                        continue;
                    }

                    let x = (&x_step_spacing * i).calc_precision(None);
                    let x_f64 = x_step_spacing_f64 * i as f64;

                    let x_uv = lerp(x_f64, &self.x_range, &(0.0..1.0)) as f32;
                    let y_uv = lerp(0.0, &self.y_range, &(1.0..0.0)) as f32;

                    let text = format!("{}", decimal_format_scientific_when(&x, text_max_digits));

                    let text_section = TextSection::Relative(
                        SectionBuilder::default()
                            .add_text(Text::new(&text).with_scale(text_size))
                            .with_screen_position((x_uv, y_uv))
                            .with_layout(
                                Layout::default_single_line()
                                    .h_align(HorizontalAlign::Center)
                                    .v_align(VerticalAlign::Top),
                            )
                            .to_owned(),
                    )
                    .into_arc_ref_cell();

                    let mut view = self.view.borrow_mut();
                    match view.add_text_section(text_section.clone(), &text_font.name) {
                        Err(_) => {
                            view.add_font(text_font.clone()).unwrap();
                            view.add_text_section(text_section.clone(), &text_font.name)
                                .unwrap();
                        }
                        _ => {}
                    }
                }
            }

            {
                let start_index = (y_range_start / y_step_spacing_f64).ceil() as i128;
                let end_index = (y_range_end / y_step_spacing_f64).floor() as i128;

                for i in start_index..=end_index {
                    if i == 0 {
                        continue;
                    }

                    let y = (&y_step_spacing * i).calc_precision(None);
                    let y_f64 = y_step_spacing_f64 * i as f64;

                    let x_uv = lerp(0.0, &self.x_range, &(0.0..1.0)) as f32;
                    let y_uv = lerp(y_f64, &self.y_range, &(1.0..0.0)) as f32;

                    let text = format!(" {}", decimal_format_scientific_when(&y, text_max_digits));

                    let text_section = TextSection::Relative(
                        SectionBuilder::default()
                            .add_text(Text::new(&text).with_scale(text_size))
                            .with_screen_position((x_uv, y_uv))
                            .with_layout(
                                Layout::default_single_line()
                                    .h_align(HorizontalAlign::Left)
                                    .v_align(VerticalAlign::Center),
                            )
                            .to_owned(),
                    )
                    .into_arc_ref_cell();

                    let mut view = self.view.borrow_mut();
                    match view.add_text_section(text_section.clone(), &text_font.name) {
                        Err(_) => {
                            view.add_font(text_font.clone()).unwrap();
                            view.add_text_section(text_section.clone(), &text_font.name)
                                .unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }

        //-----------

        println!();
    }

    fn display_function_graphs(&mut self) {
        let mut points = Vec::new();

        let sample_freq = 5000u32;

        let x_start = self.x_range.start;
        let x_len = self.x_range.end - x_start;

        let step = x_len / sample_freq as f64;

        for index in 0..self.functions.len() {
            let f = &self.functions[index];

            points.clear();
            for i in 0..=sample_freq {
                let x = x_start + (step * i as f64);
                let y = (f.function)(x, &self.parameter);

                let (sx, sy) = self.global_to_screen((x, y));

                points.push([sx, sy]);
            }

            self.vertices_add_polyline(&points, f.style.thickness, f.style.color);
        }
    }

    fn vertices_add_polyline(&mut self, points: &[[f32; 2]], width: f32, color: RGBA) {
        let mut last_point = None;
        for point in points {
            self.vertices_add_circle(*point, width / 2.0, color, 16);

            if let Some(last_point) = last_point {
                self.vertices_add_line(last_point, *point, width, color);
            }

            last_point = Some(*point);
        }
    }

    fn vertices_add_line(&mut self, end1: [f32; 2], end2: [f32; 2], width: f32, color: RGBA) {
        let color = color.into();

        let view = &mut self.view.as_ref().borrow_mut();

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

        view.append_render_vertices(&mut vec![
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

    fn vertices_add_circle(&mut self, center: [f32; 2], radius: f32, color: RGBA, resolution: u8) {
        let color = color.into();

        let view = &mut self.view.as_ref().borrow_mut();

        let scale = u8::MAX as f32 / resolution as f32;

        let mut last_point: Option<[f32; 2]> = None;
        for i in (0..=resolution).chain([0].into_iter()) {
            let index = i as f32 * scale;

            let sin_cos = Self::CIRCLE_SIN_COS_LOOKUP[index as usize];
            let sin = sin_cos[0];
            let cos = sin_cos[1];

            let x = center[0] + radius * cos;
            let y = center[1] + radius * sin;

            let point = [x, y];

            if let Some(last_point) = last_point {
                view.append_render_vertices(&mut vec![
                    Vertex {
                        position: [last_point[0], last_point[1], 0.0],
                        color,
                    },
                    Vertex {
                        position: [center[0], center[1], 0.0],
                        color,
                    },
                    Vertex {
                        position: [point[0], point[1], 0.0],
                        color,
                    },
                ]);
            }

            last_point = Some(point);
        }
    }

    const ERROR_DEC_TO_F64: &'static str = "Error while trying to map BigDecimal to f64";

    const CIRCLE_SIN_COS_LOOKUP: [[f32; 2]; 256] = [
        [0.0, 1.0],
        [0.024541229, 0.9996988],
        [0.049067676, 0.99879545],
        [0.07356457, 0.99729043],
        [0.09801714, 0.9951847],
        [0.12241068, 0.99247956],
        [0.14673047, 0.9891765],
        [0.1709619, 0.98527765],
        [0.19509032, 0.98078525],
        [0.21910124, 0.9757021],
        [0.2429802, 0.97003126],
        [0.26671278, 0.96377605],
        [0.2902847, 0.95694035],
        [0.31368175, 0.94952816],
        [0.33688986, 0.94154406],
        [0.35989505, 0.9329928],
        [0.38268346, 0.9238795],
        [0.40524134, 0.9142097],
        [0.42755508, 0.9039893],
        [0.44961134, 0.8932243],
        [0.47139674, 0.88192123],
        [0.49289823, 0.87008697],
        [0.51410276, 0.8577286],
        [0.5349976, 0.8448536],
        [0.55557024, 0.8314696],
        [0.5758082, 0.8175848],
        [0.5956993, 0.8032075],
        [0.61523163, 0.7883464],
        [0.63439333, 0.77301043],
        [0.65317285, 0.7572088],
        [0.671559, 0.7409511],
        [0.68954057, 0.7242471],
        [0.70710677, 0.70710677],
        [0.7242471, 0.6895405],
        [0.7409512, 0.6715589],
        [0.7572089, 0.6531728],
        [0.77301043, 0.6343933],
        [0.7883464, 0.6152316],
        [0.8032075, 0.5956993],
        [0.8175848, 0.57580817],
        [0.83146966, 0.5555702],
        [0.8448536, 0.53499764],
        [0.85772866, 0.5141027],
        [0.87008697, 0.49289817],
        [0.8819213, 0.47139665],
        [0.8932243, 0.4496113],
        [0.9039893, 0.4275551],
        [0.9142098, 0.40524128],
        [0.9238795, 0.38268343],
        [0.9329928, 0.35989496],
        [0.94154406, 0.33688983],
        [0.9495282, 0.31368166],
        [0.95694035, 0.29028463],
        [0.96377605, 0.26671275],
        [0.97003126, 0.24298012],
        [0.9757021, 0.21910124],
        [0.9807853, 0.19509023],
        [0.98527765, 0.17096186],
        [0.9891765, 0.1467305],
        [0.99247956, 0.122410625],
        [0.9951847, 0.098017134],
        [0.99729043, 0.07356449],
        [0.99879545, 0.04906765],
        [0.9996988, 0.024541136],
        [1.0, -4.371139e-8],
        [0.9996988, -0.024541223],
        [0.99879545, -0.04906774],
        [0.99729043, -0.073564574],
        [0.9951847, -0.09801722],
        [0.9924795, -0.12241071],
        [0.9891765, -0.14673057],
        [0.98527765, -0.17096195],
        [0.98078525, -0.19509032],
        [0.9757021, -0.21910131],
        [0.97003126, -0.2429802],
        [0.96377605, -0.26671284],
        [0.9569403, -0.29028472],
        [0.94952816, -0.31368172],
        [0.94154406, -0.33688992],
        [0.9329928, -0.35989505],
        [0.9238795, -0.38268352],
        [0.9142097, -0.40524134],
        [0.9039893, -0.42755508],
        [0.8932243, -0.44961137],
        [0.88192123, -0.47139683],
        [0.870087, -0.49289817],
        [0.8577286, -0.51410276],
        [0.8448535, -0.5349977],
        [0.83146954, -0.55557036],
        [0.8175848, -0.57580817],
        [0.8032075, -0.59569937],
        [0.78834635, -0.6152317],
        [0.7730105, -0.6343933],
        [0.7572088, -0.65317285],
        [0.74095106, -0.67155904],
        [0.724247, -0.6895407],
        [0.70710677, -0.70710677],
        [0.6895405, -0.72424716],
        [0.67155886, -0.74095124],
        [0.65317285, -0.7572088],
        [0.6343933, -0.7730105],
        [0.6152315, -0.78834647],
        [0.59569913, -0.80320764],
        [0.57580817, -0.8175848],
        [0.5555702, -0.83146966],
        [0.53499746, -0.84485364],
        [0.51410276, -0.8577286],
        [0.49289814, -0.870087],
        [0.47139663, -0.88192135],
        [0.44961137, -0.8932243],
        [0.42755505, -0.9039893],
        [0.40524122, -0.9142098],
        [0.38268328, -0.9238796],
        [0.35989505, -0.9329928],
        [0.3368898, -0.9415441],
        [0.3136816, -0.9495282],
        [0.29028472, -0.95694035],
        [0.26671273, -0.96377605],
        [0.24298008, -0.97003126],
        [0.21910107, -0.97570217],
        [0.19509031, -0.9807853],
        [0.17096181, -0.98527765],
        [0.14673033, -0.9891765],
        [0.1224107, -0.9924795],
        [0.0980171, -0.9951847],
        [0.07356445, -0.9972905],
        [0.049067486, -0.99879545],
        [0.02454121, -0.9996988],
        [-8.742278e-8, -1.0],
        [-0.024541385, -0.9996988],
        [-0.04906766, -0.99879545],
        [-0.07356462, -0.99729043],
        [-0.09801727, -0.9951847],
        [-0.12241087, -0.9924795],
        [-0.1467305, -0.9891765],
        [-0.17096199, -0.98527765],
        [-0.19509049, -0.98078525],
        [-0.21910124, -0.9757021],
        [-0.24298024, -0.97003126],
        [-0.2667129, -0.96377605],
        [-0.29028487, -0.9569403],
        [-0.31368178, -0.94952816],
        [-0.33688995, -0.94154406],
        [-0.3598952, -0.93299276],
        [-0.38268343, -0.9238795],
        [-0.4052414, -0.9142097],
        [-0.42755523, -0.90398926],
        [-0.4496115, -0.8932242],
        [-0.47139677, -0.88192123],
        [-0.4928983, -0.8700869],
        [-0.5141029, -0.85772854],
        [-0.53499764, -0.8448536],
        [-0.5555703, -0.83146954],
        [-0.57580835, -0.8175847],
        [-0.5956993, -0.8032075],
        [-0.61523163, -0.7883464],
        [-0.6343934, -0.7730104],
        [-0.65317297, -0.7572087],
        [-0.671559, -0.7409511],
        [-0.6895406, -0.72424704],
        [-0.7071069, -0.70710665],
        [-0.7242471, -0.68954057],
        [-0.7409512, -0.6715589],
        [-0.75720876, -0.6531729],
        [-0.77301043, -0.63439333],
        [-0.78834647, -0.6152316],
        [-0.8032076, -0.5956992],
        [-0.81758493, -0.57580805],
        [-0.8314698, -0.55557],
        [-0.84485376, -0.53499734],
        [-0.85772854, -0.5141028],
        [-0.87008697, -0.4928982],
        [-0.8819213, -0.47139668],
        [-0.89322436, -0.44961122],
        [-0.9039894, -0.42755494],
        [-0.91420984, -0.40524107],
        [-0.9238797, -0.38268313],
        [-0.93299276, -0.3598951],
        [-0.94154406, -0.33688986],
        [-0.9495282, -0.3136817],
        [-0.95694035, -0.29028454],
        [-0.9637761, -0.26671258],
        [-0.9700313, -0.24297991],
        [-0.9757022, -0.2191009],
        [-0.98078525, -0.19509038],
        [-0.98527765, -0.17096189],
        [-0.9891765, -0.14673041],
        [-0.99247956, -0.122410536],
        [-0.9951847, -0.09801693],
        [-0.9972905, -0.07356428],
        [-0.99879545, -0.049067326],
        [-0.9996988, -0.024541287],
        [-1.0, 1.1924881e-8],
        [-0.9996988, 0.02454131],
        [-0.99879545, 0.049067825],
        [-0.99729043, 0.07356478],
        [-0.9951847, 0.09801743],
        [-0.9924795, 0.122411035],
        [-0.9891765, 0.14673042],
        [-0.98527765, 0.17096192],
        [-0.98078525, 0.19509041],
        [-0.9757021, 0.2191014],
        [-0.9700312, 0.2429804],
        [-0.963776, 0.26671305],
        [-0.95694023, 0.29028502],
        [-0.9495282, 0.3136817],
        [-0.94154406, 0.3368899],
        [-0.93299276, 0.35989514],
        [-0.92387944, 0.3826836],
        [-0.91420966, 0.40524155],
        [-0.90398914, 0.42755538],
        [-0.8932241, 0.44961166],
        [-0.8819213, 0.4713967],
        [-0.87008697, 0.49289823],
        [-0.85772854, 0.5141028],
        [-0.84485346, 0.53499776],
        [-0.8314695, 0.5555704],
        [-0.81758463, 0.57580847],
        [-0.8032076, 0.59569925],
        [-0.7883464, 0.6152316],
        [-0.77301043, 0.63439333],
        [-0.75720876, 0.6531729],
        [-0.740951, 0.6715591],
        [-0.7242469, 0.68954074],
        [-0.70710653, 0.707107],
        [-0.6895406, 0.72424704],
        [-0.671559, 0.7409511],
        [-0.6531728, 0.7572089],
        [-0.63439316, 0.77301055],
        [-0.61523145, 0.7883465],
        [-0.5956991, 0.8032077],
        [-0.5758079, 0.817585],
        [-0.5555703, 0.8314696],
        [-0.53499764, 0.8448536],
        [-0.5141027, 0.85772866],
        [-0.49289808, 0.8700871],
        [-0.47139654, 0.88192135],
        [-0.44961107, 0.8932244],
        [-0.4275548, 0.90398943],
        [-0.40524137, 0.9142097],
        [-0.38268343, 0.92387956],
        [-0.35989496, 0.9329928],
        [-0.3368897, 0.9415441],
        [-0.31368154, 0.9495283],
        [-0.2902844, 0.9569404],
        [-0.2667124, 0.9637762],
        [-0.24298023, 0.97003126],
        [-0.21910122, 0.9757021],
        [-0.19509023, 0.9807853],
        [-0.17096172, 0.98527765],
        [-0.14673024, 0.9891766],
        [-0.12241037, 0.99247956],
        [-0.09801677, 0.9951848],
        [-0.0735646, 0.99729043],
        [-0.04906764, 0.99879545],
        [-0.024541123, 0.9996988],
    ];
}
