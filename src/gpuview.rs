use anyhow::Context;
use std::{cell::RefCell, sync::Arc};
use wgpu::util::DeviceExt;
use wgpu_text::{
    glyph_brush::{
        ab_glyph::{FontRef, FontVec},
        Layout, OwnedSection, Section, SectionBuilder, Text,
    },
    BrushBuilder, TextBrush,
};

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
pub struct FrameVertex {
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
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct TextPrimitive<'a> {
    font: &'a [u8],
    sections: Vec<Arc<RefCell<OwnedSection>>>,

    brush: Option<TextBrush<FontRef<'a>>>,

    is_initialized: bool,
}

impl<'a> TextPrimitive<'a> {
    pub fn new(font: &'a [u8], sections: Vec<Arc<RefCell<OwnedSection>>>) -> Self {
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
        let brush = BrushBuilder::using_font_bytes(&self.font)
            .context("Failed to build text brush using font bytes.")?
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
}

pub trait ShaderDescriptor {
    fn initialize(&mut self, device: &wgpu::Device);
    fn update_buffers(&self, queue: &wgpu::Queue);
    fn shader_source(&self) -> wgpu::ShaderSource;
    fn bind_group_and_layout(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout);
}

pub struct GPUView<'a> {
    width: u32,
    height: u32,

    multisample_state: wgpu::MultisampleState,
    clear_color: wgpu::Color,

    shader_descriptor: Arc<RefCell<dyn ShaderDescriptor>>,
    render_vertices: Vec<Vertex>,
    frame_vertices: Vec<FrameVertex>,

    text_primitives: Vec<TextPrimitive<'a>>,

    shader_bind_group: Option<wgpu::BindGroup>,
    render_vertices_buffer: Option<wgpu::Buffer>,
    frame_vertices_buffer: Option<wgpu::Buffer>,
    resolve_texture: Option<wgpu::Texture>,
    msaa_texture: Option<wgpu::Texture>,
    render_pipeline: Option<wgpu::RenderPipeline>,

    frame_bind_group: Option<wgpu::BindGroup>,

    is_initialized: bool,
    render_vertices_changed: bool,
    frame_vertices_changed: bool,
}

impl<'a> GPUView<'a> {
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

    pub fn new(
        width: u32,
        height: u32,
        shader_descriptor: Arc<RefCell<dyn ShaderDescriptor>>,
        frame_vertices: Vec<FrameVertex>,
    ) -> Self {
        let multisample_state = wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        let clear_color = wgpu::Color::TRANSPARENT;

        Self {
            width,
            height,
            multisample_state,
            clear_color,
            shader_descriptor,
            render_vertices: Vec::new(),
            frame_vertices,
            text_primitives: Vec::new(),
            shader_bind_group: None,
            render_vertices_buffer: None,
            frame_vertices_buffer: None,
            msaa_texture: None,
            resolve_texture: None,
            render_pipeline: None,
            frame_bind_group: None,
            is_initialized: false,
            render_vertices_changed: false,
            frame_vertices_changed: false,
        }
    }

    pub fn new_rect_frame(
        width: u32,
        height: u32,
        shader_descriptor: Arc<RefCell<dyn ShaderDescriptor>>,
        frame_vertice_upper_left: [f32; 2],
        frame_vertice_lower_right: [f32; 2],
    ) -> Self {
        let frame_vertices = vec![
            FrameVertex {
                // A
                position: frame_vertice_upper_left,
                tex_coords: [0.0, 0.0],
            },
            FrameVertex {
                // B
                position: [frame_vertice_upper_left[0], frame_vertice_lower_right[1]],
                tex_coords: [0.0, 1.0],
            },
            FrameVertex {
                // C
                position: frame_vertice_lower_right,
                tex_coords: [1.0, 1.0],
            },
            FrameVertex {
                // A
                position: frame_vertice_upper_left,
                tex_coords: [0.0, 0.0],
            },
            FrameVertex {
                // C
                position: frame_vertice_lower_right,
                tex_coords: [1.0, 1.0],
            },
            FrameVertex {
                // D
                position: [frame_vertice_lower_right[0], frame_vertice_upper_left[1]],
                tex_coords: [1.0, 0.0],
            },
        ];

        Self::new(width, height, shader_descriptor, frame_vertices)
    }

    pub fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(self))
    }

    pub fn set_multisample_state(&mut self, multisample_state: wgpu::MultisampleState) {
        self.multisample_state = multisample_state;
    }

    pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
        self.clear_color = clear_color;
    }

    pub fn set_render_vertices(&mut self, vertices: Vec<Vertex>) {
        self.render_vertices = vertices;
        self.render_vertices_changed = true;
    }

    pub fn set_text_primitives(&mut self, text_primitives: Vec<TextPrimitive<'a>>) {
        self.text_primitives = text_primitives;
    }

    pub fn initialize(&mut self, device: &wgpu::Device) -> anyhow::Result<()> {
        self.shader_descriptor.borrow_mut().initialize(device);

        let (shader_bind_group, shader_bind_group_layout) = self
            .shader_descriptor
            .borrow()
            .bind_group_and_layout(device);

        let render_vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GPUView Render Vertices Buffer"),
            contents: bytemuck::cast_slice(self.render_vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let frame_vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GPUView Frame Vertices Buffer"),
            contents: bytemuck::cast_slice(self.frame_vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let resolve_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPUView Resolve Texture"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
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
                width: self.width,
                height: self.height,
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
                entry_point: "vs_main",
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
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        self.shader_bind_group = Some(shader_bind_group);
        self.render_vertices_buffer = Some(render_vertices_buffer);
        self.frame_vertices_buffer = Some(frame_vertices_buffer);
        self.resolve_texture = Some(resolve_texture);
        self.msaa_texture = Some(msaa_texture);
        self.render_pipeline = Some(render_pipeline);
        self.frame_bind_group = Some(frame_bind_group);
        self.is_initialized = true;

        Ok(())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32, device: &wgpu::Device) {
        self.width = new_width;
        self.height = new_height;

        if !self.is_initialized {
            return;
        }

        self.resolve_texture.as_ref().unwrap().destroy();
        self.resolve_texture = Some(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPUView Resolve Texture"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }));

        self.msaa_texture.as_ref().unwrap().destroy();
        self.msaa_texture = Some(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPUView MSAA Texture"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.multisample_state.count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }));
    }

    pub fn update_vertice_buffers(&mut self, device: &wgpu::Device) {
        if self.render_vertices_changed {
            self.render_vertices_buffer.as_ref().unwrap().destroy();

            self.render_vertices_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("GPUView Render Vertices Buffer"),
                    contents: bytemuck::cast_slice(self.render_vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));

            self.render_vertices_changed = false;
        }

        if self.frame_vertices_changed {
            self.frame_vertices_buffer.as_ref().unwrap().destroy();

            self.frame_vertices_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("GPUView Frame Vertices Buffer"),
                    contents: bytemuck::cast_slice(self.frame_vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));

            self.frame_vertices_changed = false;
        }
    }
}

pub struct GPUMultiView<'a> {
    clear_color: wgpu::Color,

    render_views: Vec<Arc<RefCell<GPUView<'a>>>>,
    text_primitives: Vec<TextPrimitive<'a>>,

    surface: Option<wgpu::Surface<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,

    is_initialized: bool,
}

impl<'a> GPUMultiView<'a> {
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

        let shader = device.create_shader_module(wgpu::include_wgsl!("multiview.wgsl"));

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
                entry_point: "vs_main",
                buffers: &[FrameVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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
        });

        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.render_pipeline = Some(render_pipeline);
        self.is_initialized = true;
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32, device: &wgpu::Device) {
        if !self.is_initialized {
            return;
        }

        let surface_config = self.surface_config.as_mut().unwrap();

        surface_config.width = new_width;
        surface_config.height = new_height;

        self.surface
            .as_ref()
            .unwrap()
            .configure(device, surface_config);
    }

    pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
        self.clear_color = clear_color;
    }

    pub fn set_render_views(&mut self, views: Vec<Arc<RefCell<GPUView<'a>>>>) {
        self.render_views = views;
    }

    pub fn set_text_primitives(&mut self, text_primitives: Vec<TextPrimitive<'a>>) {
        self.text_primitives = text_primitives;
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.is_initialized {
            return;
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

        {
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

        for render_view in &self.render_views {
            if !render_view.borrow().is_initialized {
                render_view.borrow_mut().initialize(device).unwrap();
            }

            let render_width = render_view.borrow().width;
            let render_height = render_view.borrow().height;
            let multisample_state = render_view.borrow().multisample_state;

            render_view.borrow_mut().update_vertice_buffers(device);

            let resolve_texture_view = render_view
                .borrow()
                .resolve_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor::default());

            for text_primitive in &mut render_view.borrow_mut().text_primitives {
                if !text_primitive.is_initialized {
                    text_primitive
                        .initialize(device, render_width, render_height, multisample_state)
                        .unwrap();
                }

                let sections = text_primitive
                    .sections
                    .iter()
                    .map(|section| section.borrow().clone())
                    .collect::<Vec<_>>();

                let sections = sections.iter().map(|section| section).collect::<Vec<_>>();

                text_primitive
                    .brush
                    .as_mut()
                    .unwrap()
                    .queue(device, queue, sections)
                    .unwrap();
            }

            let render_view_ref = render_view.borrow();

            {
                let shader_bind_group = render_view_ref.shader_bind_group.as_ref().unwrap();

                let msaa_texture_view = render_view_ref
                    .msaa_texture
                    .as_ref()
                    .unwrap()
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let render_pipeline = render_view_ref.render_pipeline.as_ref().unwrap();

                let render_vertices_buffer =
                    render_view_ref.render_vertices_buffer.as_ref().unwrap();

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("GPUView Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &msaa_texture_view,
                        resolve_target: Some(&resolve_texture_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(render_view_ref.clear_color),
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
                render_pass.draw(0..render_view.borrow().render_vertices.len() as u32, 0..1);

                for text_primitive in &render_view_ref.text_primitives {
                    text_primitive
                        .brush
                        .as_ref()
                        .unwrap()
                        .draw(&mut render_pass);
                }
            }

            {
                let frame_bind_group = render_view_ref.frame_bind_group.as_ref().unwrap();

                let frame_vertices_buffer = render_view_ref.frame_vertices_buffer.as_ref().unwrap();

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
                render_pass.draw(0..render_view.borrow().frame_vertices.len() as u32, 0..1);
            }
        }

        let render_width = self.width().unwrap();
        let render_height = self.height().unwrap();

        for text_primitive in &mut self.text_primitives {
            if !text_primitive.is_initialized {
                text_primitive
                    .initialize(
                        device,
                        render_width,
                        render_height,
                        wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                    )
                    .unwrap();
            }

            let sections = text_primitive
                .sections
                .iter()
                .map(|section| section.borrow().clone())
                .collect::<Vec<_>>();

            let sections = sections.iter().map(|section| section).collect::<Vec<_>>();

            text_primitive
                .brush
                .as_mut()
                .unwrap()
                .queue(device, queue, sections)
                .unwrap();
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

        queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }
}
