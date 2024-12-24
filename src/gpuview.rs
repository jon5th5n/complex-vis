use anyhow::Context;
use std::{
    cell::RefCell,
    ops::{Add, Div, Mul, Range, Sub},
    sync::Arc,
};
use wgpu::util::DeviceExt;
use wgpu_text::{
    glyph_brush::{
        ab_glyph::{FontArc, FontRef, FontVec},
        OwnedSection,
    },
    BrushBuilder, TextBrush,
};

use crate::math::lerp;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FrameVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
impl FrameVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FrameVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    fn vertices_from_rect(upper_left: (f32, f32), lower_right: (f32, f32)) -> Vec<FrameVertex> {
        vec![
            FrameVertex {
                // A
                position: [upper_left.0, upper_left.1],
                tex_coords: [0.0, 0.0],
            },
            FrameVertex {
                // B
                position: [upper_left.0, lower_right.1],
                tex_coords: [0.0, 1.0],
            },
            FrameVertex {
                // C
                position: [lower_right.0, lower_right.1],
                tex_coords: [1.0, 1.0],
            },
            FrameVertex {
                // A
                position: [upper_left.0, upper_left.1],
                tex_coords: [0.0, 0.0],
            },
            FrameVertex {
                // C
                position: [lower_right.0, lower_right.1],
                tex_coords: [1.0, 1.0],
            },
            FrameVertex {
                // D
                position: [lower_right.0, upper_left.1],
                tex_coords: [1.0, 0.0],
            },
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GPUViewFrame {
    Whole,
    UpperLeftQuad,
    UpperRightQuad,
    LowerLeftQuad,
    LowerRightQuad,
    Custom {
        upper_left: (f32, f32),
        lower_right: (f32, f32),
    },
}

impl GPUViewFrame {
    pub fn with_margin(self, margin: (f32, f32)) -> Self {
        let upper_left = self.upper_left();
        let lower_right = self.lower_right();

        let upper_left = (upper_left.0 + margin.0, upper_left.1 - margin.1);
        let lower_right = (lower_right.0 - margin.0, lower_right.1 + margin.1);

        Self::Custom {
            upper_left,
            lower_right,
        }
    }

    // 0---1---2
    // |   |   |
    // 3---4---5
    // |   |   |
    // 6---7---8
    const QUAD_VERTS_POS: [(f32, f32); 9] = [
        (-1.0, 1.0),  // 0
        (0.0, 1.0),   // 1
        (1.0, 1.0),   // 2
        (-1.0, 0.0),  // 3
        (0.0, 0.0),   // 4
        (1.0, 0.0),   // 5
        (-1.0, -1.0), // 6
        (0.0, -1.0),  // 7
        (1.0, -1.0),  // 8
    ];

    fn upper_left(&self) -> (f32, f32) {
        match self {
            GPUViewFrame::Whole => Self::QUAD_VERTS_POS[0],
            GPUViewFrame::UpperLeftQuad => Self::QUAD_VERTS_POS[0],
            GPUViewFrame::UpperRightQuad => Self::QUAD_VERTS_POS[1],
            GPUViewFrame::LowerLeftQuad => Self::QUAD_VERTS_POS[3],
            GPUViewFrame::LowerRightQuad => Self::QUAD_VERTS_POS[4],
            GPUViewFrame::Custom {
                upper_left,
                lower_right: _,
            } => *upper_left,
        }
    }

    fn lower_right(&self) -> (f32, f32) {
        match self {
            GPUViewFrame::Whole => Self::QUAD_VERTS_POS[8],
            GPUViewFrame::UpperLeftQuad => Self::QUAD_VERTS_POS[4],
            GPUViewFrame::UpperRightQuad => Self::QUAD_VERTS_POS[5],
            GPUViewFrame::LowerLeftQuad => Self::QUAD_VERTS_POS[7],
            GPUViewFrame::LowerRightQuad => Self::QUAD_VERTS_POS[8],
            GPUViewFrame::Custom {
                upper_left: _,
                lower_right,
            } => *lower_right,
        }
    }

    fn frame_vertices(&self) -> Vec<FrameVertex> {
        FrameVertex::vertices_from_rect(self.upper_left(), self.lower_right())
    }

    fn relative_dimensions(&self) -> (f32, f32) {
        match self {
            GPUViewFrame::Whole => (1.0, 1.0),
            GPUViewFrame::UpperLeftQuad => (0.5, 0.5),
            GPUViewFrame::UpperRightQuad => (0.5, 0.5),
            GPUViewFrame::LowerLeftQuad => (0.5, 0.5),
            GPUViewFrame::LowerRightQuad => (0.5, 0.5),
            GPUViewFrame::Custom {
                upper_left,
                lower_right,
            } => (lower_right.0 - upper_left.0, upper_left.1 - lower_right.1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    pub name: String,
    pub font: FontArc,
}

#[derive(Debug)]
pub enum TextSection {
    Absolute(OwnedSection),
    Relative(OwnedSection),
}

impl TextSection {
    pub fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(self))
    }

    fn create_section(&self, render_width: u32, render_height: u32) -> OwnedSection {
        match self {
            TextSection::Absolute(section) => section.clone(),
            TextSection::Relative(section) => {
                let relative_pos = section.screen_position;
                section.clone().with_screen_position((
                    relative_pos.0 * render_width as f32,
                    relative_pos.1 * render_height as f32,
                ))
            }
        }
    }
}

pub struct TextPrimitive {
    font: Font,
    sections: Vec<Arc<RefCell<TextSection>>>,

    brush: Option<TextBrush<FontArc>>,

    is_initialized: bool,
}

impl TextPrimitive {
    pub fn new(font: Font, sections: Vec<Arc<RefCell<TextSection>>>) -> Self {
        Self {
            font,
            sections,
            brush: None,
            is_initialized: false,
        }
    }

    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        render_width: u32,
        render_height: u32,
        multisample_state: wgpu::MultisampleState,
    ) -> anyhow::Result<()> {
        let brush = BrushBuilder::using_font(self.font.font.clone())
            .with_multisample(multisample_state)
            .build(
                device,
                render_width,
                render_height,
                wgpu::TextureFormat::Bgra8Unorm,
            );

        self.brush = Some(brush);
        self.is_initialized = true;

        Ok(())
    }

    fn create_sections(&self, render_width: u32, render_height: u32) -> Vec<OwnedSection> {
        self.sections
            .iter()
            .map(|section| section.borrow().create_section(render_width, render_height))
            .collect::<Vec<_>>()
    }
}

pub trait ShaderDescriptor {
    fn initialize(&mut self, device: &wgpu::Device) -> anyhow::Result<()>;
    fn update_buffers(&mut self, queue: &wgpu::Queue) -> anyhow::Result<()>;
    fn shader_source(&self) -> wgpu::ShaderSource;
    fn bind_group_and_layout(
        &self,
        device: &wgpu::Device,
    ) -> anyhow::Result<(wgpu::BindGroup, wgpu::BindGroupLayout)>;
}

pub struct GPUView {
    frame: GPUViewFrame,

    multisample_state: wgpu::MultisampleState,
    clear_color: wgpu::Color,

    shader_descriptor: Arc<RefCell<dyn ShaderDescriptor>>,
    render_vertices: Vec<Vertex>,

    text_primitives: Vec<TextPrimitive>,

    texture_width: Option<u32>,
    texture_height: Option<u32>,
    resolve_texture: Option<wgpu::Texture>,
    msaa_texture: Option<wgpu::Texture>,

    shader_bind_group: Option<wgpu::BindGroup>,
    render_vertices_buffer: Option<wgpu::Buffer>,
    frame_vertices_buffer: Option<wgpu::Buffer>,
    render_pipeline: Option<wgpu::RenderPipeline>,

    resolve_texture_sampler: Option<wgpu::Sampler>,
    frame_bind_group_layout: Option<wgpu::BindGroupLayout>,
    frame_bind_group: Option<wgpu::BindGroup>,

    is_initialized: bool,
    render_vertices_changed: bool,
    frame_changed: bool,
}

impl GPUView {
    const FRAME_BIND_GROUP_LAYOUT_DESCIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("GPUView Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

    pub fn new(frame: GPUViewFrame, shader_descriptor: Arc<RefCell<dyn ShaderDescriptor>>) -> Self {
        let multisample_state = wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        let clear_color = wgpu::Color::TRANSPARENT;

        Self {
            frame,
            multisample_state,
            clear_color,
            shader_descriptor,
            render_vertices: Vec::new(),
            text_primitives: Vec::new(),
            texture_width: None,
            texture_height: None,
            msaa_texture: None,
            resolve_texture: None,
            shader_bind_group: None,
            render_vertices_buffer: None,
            frame_vertices_buffer: None,
            render_pipeline: None,
            resolve_texture_sampler: None,
            frame_bind_group_layout: None,
            frame_bind_group: None,
            is_initialized: false,
            render_vertices_changed: false,
            frame_changed: false,
        }
    }

    pub fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(self))
    }

    pub fn set_frame(
        &mut self,
        frame: GPUViewFrame,
        multiview: &GPUMultiView,
        device: &wgpu::Device,
    ) {
        self.frame = frame;
        self.frame_changed = true;

        let _ = self.resize(multiview, device);
    }

    pub fn set_multisample_state(&mut self, multisample_state: wgpu::MultisampleState) {
        self.multisample_state = multisample_state;
    }

    pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
        self.clear_color = clear_color;
    }

    pub fn clear_render_vertices(&mut self) {
        self.render_vertices.clear();
        self.render_vertices_changed;
    }

    pub fn set_render_vertices(&mut self, vertices: Vec<Vertex>) {
        self.render_vertices = vertices;
        self.render_vertices_changed = true;
    }

    pub fn append_render_vertices(&mut self, vertices: &mut Vec<Vertex>) {
        self.render_vertices.append(vertices);
        self.render_vertices_changed = true;
    }

    pub fn get_render_vertices_len(&self) -> usize {
        self.render_vertices.len()
    }

    pub fn clear_text_sections_and_fonts(&mut self) {
        self.text_primitives.clear();
    }

    pub fn clear_text_sections(&mut self) {
        for primitive in &mut self.text_primitives {
            primitive.sections.clear();
        }
    }

    pub fn add_font(&mut self, font: Font) -> anyhow::Result<()> {
        if self
            .text_primitives
            .iter()
            .any(|p| p.font.name == font.name)
        {
            return Err(anyhow::Error::msg(
                "Fonts have to be uniquely identifyable by their name.",
            ));
        }

        let text_primitive = TextPrimitive::new(font, Vec::new());
        self.text_primitives.push(text_primitive);

        Ok(())
    }

    pub fn set_fonts(&mut self, fonts: Vec<Font>) -> anyhow::Result<()> {
        self.clear_text_sections_and_fonts();

        for font in fonts {
            self.add_font(font)?;
        }

        Ok(())
    }

    pub fn add_text_section(
        &mut self,
        text_section: Arc<RefCell<TextSection>>,
        font_name: &str,
    ) -> anyhow::Result<()> {
        let text_primitive = self
            .text_primitives
            .iter_mut()
            .find(|p| p.font.name == font_name)
            .context("Specified font was not added yet.")?;

        text_primitive.sections.push(text_section);

        Ok(())
    }

    pub fn initialize(
        &mut self,
        multiview: &GPUMultiView,
        device: &wgpu::Device,
    ) -> anyhow::Result<()> {
        self.shader_descriptor.borrow_mut().initialize(device)?;

        let (shader_bind_group, shader_bind_group_layout) = self
            .shader_descriptor
            .borrow()
            .bind_group_and_layout(device)?;

        let render_vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GPUView Render Vertices Buffer"),
            contents: bytemuck::cast_slice(self.render_vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let frame_vertices = self.frame.frame_vertices();

        let frame_vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GPUView Frame Vertices Buffer"),
            contents: bytemuck::cast_slice(frame_vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let multiview_width = multiview
            .width()
            .context("Provided multiview was not initialized correctly.")?;
        let multiview_height = multiview
            .height()
            .context("Provided multiview was not initialized correctly.")?;
        let (frame_relative_width, frame_relative_height) = self.frame.relative_dimensions();

        let texture_width = (multiview_width as f32 * frame_relative_width) as u32;
        let texture_height = (multiview_height as f32 * frame_relative_height) as u32;

        let resolve_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPUView Resolve Texture"),
            size: wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPUView MSAA Texture"),
            size: wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.multisample_state.count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let resolve_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let frame_bind_group_layout =
            device.create_bind_group_layout(&Self::FRAME_BIND_GROUP_LAYOUT_DESCIPTOR);

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GPUView Frame Bind Group"),
            layout: &frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &resolve_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&resolve_texture_sampler),
                },
            ],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPUView Shader Module"),
            source: self.shader_descriptor.borrow().shader_source(),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GPUView Pipeline Layout"),
            bind_group_layouts: &[&shader_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GPUView Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: self.multisample_state,
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        self.texture_width = Some(texture_width);
        self.texture_height = Some(texture_height);
        self.resolve_texture = Some(resolve_texture);
        self.msaa_texture = Some(msaa_texture);
        self.shader_bind_group = Some(shader_bind_group);
        self.render_vertices_buffer = Some(render_vertices_buffer);
        self.frame_vertices_buffer = Some(frame_vertices_buffer);
        self.render_pipeline = Some(render_pipeline);
        self.resolve_texture_sampler = Some(resolve_texture_sampler);
        self.frame_bind_group_layout = Some(frame_bind_group_layout);
        self.frame_bind_group = Some(frame_bind_group);
        self.is_initialized = true;

        Ok(())
    }

    pub fn resize(
        &mut self,
        multiview: &GPUMultiView,
        device: &wgpu::Device,
    ) -> anyhow::Result<()> {
        if !self.is_initialized || !multiview.is_initialized {
            return Err(anyhow::Error::msg(
                "Cannot resize uninitialized view or with uninitialized multiview.",
            ));
        }

        let (frame_relative_width, frame_relative_height) = self.frame.relative_dimensions();

        let texture_width = (multiview.width().unwrap() as f32 * frame_relative_width) as u32;
        let texture_height = (multiview.height().unwrap() as f32 * frame_relative_height) as u32;

        let resolve_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPUView Resolve Texture"),
            size: wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPUView MSAA Texture"),
            size: wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.multisample_state.count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GPUView Frame Bind Group"),
            layout: self.frame_bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &resolve_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        self.resolve_texture_sampler.as_ref().unwrap(),
                    ),
                },
            ],
        });

        self.resolve_texture.as_ref().unwrap().destroy();
        self.msaa_texture.as_ref().unwrap().destroy();

        self.texture_width = Some(texture_width);
        self.texture_height = Some(texture_height);
        self.resolve_texture = Some(resolve_texture);
        self.msaa_texture = Some(msaa_texture);
        self.frame_bind_group = Some(frame_bind_group);

        for text_primitive in &mut self.text_primitives {
            text_primitive.initialize(
                device,
                texture_width,
                texture_height,
                self.multisample_state,
            )?;
        }

        Ok(())
    }

    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()> {
        self.shader_descriptor.borrow_mut().update_buffers(queue)?;

        if !self.is_initialized {
            return Err(anyhow::Error::msg(
                "Cannot update vertex buffers of uninitialized view.",
            ));
        }

        if self.render_vertices_changed {
            let new_data = bytemuck::cast_slice(self.render_vertices.as_slice());

            let buffer = self.render_vertices_buffer.as_ref().unwrap();

            match buffer.size() as usize == new_data.len() {
                true => {
                    queue.write_buffer(buffer, 0, new_data);
                }
                false => {
                    buffer.destroy();
                    self.render_vertices_buffer = Some(device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("GPUView Render Vertices Buffer"),
                            contents: new_data,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        },
                    ));
                }
            }

            self.render_vertices_changed = false;
        }

        if self.frame_changed {
            let frame_vertices = self.frame.frame_vertices();
            let new_data = bytemuck::cast_slice(frame_vertices.as_slice());

            let buffer = self.frame_vertices_buffer.as_ref().unwrap();

            match buffer.size() as usize == new_data.len() {
                true => {
                    queue.write_buffer(buffer, 0, new_data);
                }
                false => {
                    buffer.destroy();
                    self.frame_vertices_buffer = Some(device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("GPUView Render Vertices Buffer"),
                            contents: new_data,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        },
                    ));
                }
            }

            self.frame_changed = false;
        }

        Ok(())
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()> {
        if !self.is_initialized {
            return Err(anyhow::Error::msg("Cannot render uninitialized view."));
        }

        self.update_buffers(device, queue)?;

        let render_width = self.texture_width.unwrap();
        let render_height = self.texture_height.unwrap();

        let resolve_texture_view = self
            .resolve_texture
            .as_ref()
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());

        for text_primitive in &mut self.text_primitives {
            if !text_primitive.is_initialized {
                text_primitive.initialize(
                    device,
                    render_width,
                    render_height,
                    self.multisample_state,
                )?;
            }

            let sections = text_primitive.create_sections(render_width, render_height);
            let sections = sections.iter().map(|section| section).collect::<Vec<_>>();

            text_primitive
                .brush
                .as_mut()
                .unwrap()
                .queue(device, queue, sections)
                .unwrap();
        }

        {
            let shader_bind_group = self.shader_bind_group.as_ref().unwrap();

            let msaa_texture_view = self
                .msaa_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor::default());

            let render_pipeline = self.render_pipeline.as_ref().unwrap();

            let render_vertices_buffer = self.render_vertices_buffer.as_ref().unwrap();

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GPUView Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_texture_view,
                    resolve_target: Some(&resolve_texture_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(render_pipeline);
            render_pass.set_bind_group(0, shader_bind_group, &[]);
            render_pass.set_vertex_buffer(0, render_vertices_buffer.slice(..));
            render_pass.draw(0..self.render_vertices.len() as u32, 0..1);

            for text_primitive in &self.text_primitives {
                text_primitive
                    .brush
                    .as_ref()
                    .unwrap()
                    .draw(&mut render_pass);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewCoordinates {
    pub view_index: usize,
    pub coordinates: (f32, f32),
}

pub struct GPUMultiView<'a> {
    clear_color: wgpu::Color,

    render_views: Vec<Arc<RefCell<GPUView>>>,

    text_primitives: Vec<TextPrimitive>,

    surface: Option<wgpu::Surface<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,

    is_initialized: bool,
}

impl<'a> GPUMultiView<'a> {
    const SHADER: &'static str = r#"
        struct VertexInput {
            @location(0) position: vec2<f32>,
            @location(1) tex_coords: vec2<f32>,
        }
        
        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) tex_coords: vec2<f32>,
        };
        
        @vertex
        fn vs_main(
            model: VertexInput,
        ) -> VertexOutput {
            var out: VertexOutput;
            out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
            out.tex_coords = model.tex_coords;
            return out;
        }
        
        @group(0) @binding(0)
        var texture: texture_2d<f32>;
        
        @group(0) @binding(1)
        var texture_sampler: sampler;
        
        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            let color = textureSample(texture, texture_sampler, in.tex_coords);
            return color;
        }
    "#;

    pub fn new() -> Self {
        let clear_color = wgpu::Color::TRANSPARENT;

        Self {
            clear_color,
            render_views: Vec::new(),
            text_primitives: Vec::new(),
            surface: None,
            surface_config: None,
            render_pipeline: None,
            is_initialized: false,
        }
    }

    pub fn width(&self) -> Option<u32> {
        Some(self.surface_config.as_ref()?.width)
    }

    pub fn height(&self) -> Option<u32> {
        Some(self.surface_config.as_ref()?.height)
    }

    pub fn initialize(
        &mut self,
        surface: wgpu::Surface<'a>,
        surface_config: wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
    ) {
        let bind_group_layout =
            device.create_bind_group_layout(&GPUView::FRAME_BIND_GROUP_LAYOUT_DESCIPTOR);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Multiview Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Self::SHADER.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[FrameVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.render_pipeline = Some(render_pipeline);
        self.is_initialized = true;
    }

    pub fn resize(
        &mut self,
        new_width: u32,
        new_height: u32,
        device: &wgpu::Device,
    ) -> anyhow::Result<()> {
        if !self.is_initialized {
            return Err(anyhow::Error::msg("Cannot resize uninitialized multiview."));
        }

        let surface_config = self.surface_config.as_mut().unwrap();

        surface_config.width = new_width;
        surface_config.height = new_height;

        self.surface
            .as_ref()
            .unwrap()
            .configure(device, surface_config);

        for render_view in &self.render_views {
            render_view.borrow_mut().resize(self, device)?;
        }

        for text_primitive in &mut self.text_primitives {
            text_primitive.initialize(
                device,
                new_width,
                new_height,
                wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            )?;
        }

        Ok(())
    }

    pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
        self.clear_color = clear_color;
    }

    pub fn set_render_views(&mut self, views: Vec<Arc<RefCell<GPUView>>>) {
        self.render_views = views;
    }

    pub fn clear_text_sections_and_fonts(&mut self) {
        self.text_primitives.clear();
    }

    pub fn clear_text_sections(&mut self) {
        for primitive in &mut self.text_primitives {
            primitive.sections.clear();
        }
    }

    pub fn add_font(&mut self, font: Font) -> anyhow::Result<()> {
        if self
            .text_primitives
            .iter()
            .any(|p| p.font.name == font.name)
        {
            return Err(anyhow::Error::msg(
                "Fonts have to be uniquely identifyable by their name.",
            ));
        }

        let text_primitive = TextPrimitive::new(font, Vec::new());
        self.text_primitives.push(text_primitive);

        Ok(())
    }

    pub fn set_fonts(&mut self, fonts: Vec<Font>) -> anyhow::Result<()> {
        self.clear_text_sections_and_fonts();

        for font in fonts {
            self.add_font(font)?;
        }

        Ok(())
    }

    pub fn add_text_section(
        &mut self,
        text_section: Arc<RefCell<TextSection>>,
        font_name: &str,
    ) -> anyhow::Result<()> {
        let text_primitive = self
            .text_primitives
            .iter_mut()
            .find(|p| p.font.name == font_name)
            .context("Specified font was not added yet.")?;

        text_primitive.sections.push(text_section);

        Ok(())
    }

    pub fn get_view_coords_behind(&self, point: (f32, f32)) -> Option<ViewCoordinates> {
        let x = point.0;
        let y = point.1;

        let mut view_coords = None;
        for i in 0..self.render_views.len() {
            let view = self.render_views[i].borrow();

            let frame_ul = view.frame.upper_left();
            let frame_lr = view.frame.lower_right();

            let view_x = lerp(x, &(frame_ul.0..frame_lr.0), &(-1.0..1.0));
            let view_y = lerp(y, &(frame_ul.1..frame_lr.1), &(1.0..-1.0));

            if view_x >= -1.0 && view_x <= 1.0 && view_y >= -1.0 && view_y <= 1.0 {
                view_coords = Some(ViewCoordinates {
                    view_index: i,
                    coordinates: (view_x, view_y),
                })
            }
        }

        view_coords
    }

    fn clear_surface(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let _clear_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("GPUView Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    fn render_view(
        &self,
        render_view: &GPUView,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> anyhow::Result<()> {
        if !self.is_initialized || !render_view.is_initialized {
            return Err(anyhow::Error::msg(
                "Cannot render (uninitialized) view on (uninitialized) multiview.",
            ));
        }

        let frame_bind_group = render_view.frame_bind_group.as_ref().unwrap();

        let frame_vertices_buffer = render_view.frame_vertices_buffer.as_ref().unwrap();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Multiview Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
        render_pass.set_bind_group(0, frame_bind_group, &[]);
        render_pass.set_vertex_buffer(0, frame_vertices_buffer.slice(..));
        render_pass.draw(0..render_view.frame.frame_vertices().len() as u32, 0..1);

        Ok(())
    }

    fn render_text(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()> {
        if !self.is_initialized {
            return Err(anyhow::Error::msg(
                "Cannot render text of uninitialized multiview.",
            ));
        }

        let render_width = self.width().unwrap();
        let render_height = self.height().unwrap();

        for text_primitive in &mut self.text_primitives {
            if !text_primitive.is_initialized {
                text_primitive.initialize(
                    device,
                    render_width,
                    render_height,
                    wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                )?;
            }

            let sections = text_primitive.create_sections(render_width, render_height);
            let sections = sections.iter().map(|section| section).collect::<Vec<_>>();

            text_primitive
                .brush
                .as_mut()
                .unwrap()
                .queue(device, queue, sections)?;
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GPUView Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for text_primitive in &self.text_primitives {
                text_primitive
                    .brush
                    .as_ref()
                    .unwrap()
                    .draw(&mut render_pass);
            }
        }

        Ok(())
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        if !self.is_initialized {
            return Err(anyhow::Error::msg("Cannot render uninitialized multiview."));
        }

        let output = self
            .surface
            .as_ref()
            .unwrap()
            .get_current_texture()
            .unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        self.clear_surface(&view, &mut encoder);

        for render_view in &self.render_views {
            if !render_view.borrow().is_initialized {
                render_view.borrow_mut().initialize(self, device)?;
            }

            // render_view.borrow_mut().update_buffers(device, queue)?;

            render_view
                .borrow_mut()
                .render(&mut encoder, device, queue)?;
            self.render_view(&render_view.borrow(), &view, &mut encoder)?;
        }

        self.render_text(&view, &mut encoder, device, queue)?;

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
