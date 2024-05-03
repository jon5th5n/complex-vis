use anyhow::Context;
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct RectDescriptor {
    pub upper_left: (f32, f32),
    pub lower_rigth: (f32, f32),
}

pub struct RenderConfig {
    pub displays_vertices: Vec<Vec<Vertex>>,
    pub displays_indices: Vec<Vec<u16>>,
}

impl RenderConfig {
    pub fn new_rects(rects: &[RectDescriptor]) -> Self {
        let displays_vertices = rects
            .into_iter()
            .map(|rect| {
                vec![
                    Vertex {
                        position: [rect.upper_left.0, rect.upper_left.1, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [rect.upper_left.0, rect.lower_rigth.1, 0.0],
                        tex_coords: [0.0, 1.0],
                    },
                    Vertex {
                        position: [rect.lower_rigth.0, rect.lower_rigth.1, 0.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [rect.lower_rigth.0, rect.upper_left.1, 0.0],
                        tex_coords: [1.0, 0.0],
                    },
                ]
            })
            .collect::<Vec<_>>();

        let displays_indices = vec![vec![0, 1, 3, 3, 1, 2]; rects.len()];

        Self {
            displays_vertices,
            displays_indices,
        }
    }
}

pub struct GPURenderer<'a> {
    // Window
    window: Arc<winit::window::Window>,
    window_size: winit::dpi::PhysicalSize<u32>,

    // GPU Handle
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // GPU Renderer
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,

    render_config: RenderConfig,

    texture_sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    render_pipeline: wgpu::RenderPipeline,
}

impl GPURenderer<'_> {
    const RENDER_VERTEX_SHADER: &'static str = r#"
        struct VertexInput {
            @location(0) position: vec3<f32>,
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
            out.clip_position = vec4<f32>(model.position, 1.0);
            out.tex_coords = model.tex_coords;
            return out;
        }
    "#;

    const RENDER_FRAGMENT_SHADER: &'static str = r#"
        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) tex_coords: vec2<f32>,
        };
        
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

    pub async fn new(
        window: winit::window::Window,
        render_config: RenderConfig,
    ) -> anyhow::Result<Self> {
        let window = Arc::new(window);

        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("GPU Adapter Request Failed.")?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f == &wgpu::TextureFormat::Bgra8Unorm)
            .context("Surface format bgra8unorm which is critical for this application is not supported by surface.")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: surface_format.required_features()
                        | wgpu::Features::BGRA8UNORM_STORAGE
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        | wgpu::Features::TEXTURE_BINDING_ARRAY
                        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                        | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
                        | wgpu::Features::CLEAR_TEXTURE,
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: Some("Renderer Created Device"),
                },
                None, // Trace path
            )
            .await
            .context("GPU Device Request Failed.")?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Renderer Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: std::num::NonZeroU32::new(1), // !TODO
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Renderer Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(Self::RENDER_VERTEX_SHADER.into()),
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Renderer Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(Self::RENDER_FRAGMENT_SHADER.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Renderer Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Renderer Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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

        Ok(Self {
            window,
            window_size,
            device,
            queue,
            surface,
            surface_config,
            render_config,
            texture_sampler,
            texture_bind_group_layout,
            render_pipeline,
        })
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window_size
    }

    pub fn window_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.window_size = new_size;

        self.surface_config.width = self.window_size.width;
        self.surface_config.height = self.window_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn device_arc(&self) -> Arc<wgpu::Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn queue_arc(&self) -> Arc<wgpu::Queue> {
        self.queue.clone()
    }

    pub fn render(&mut self, textures: Vec<&wgpu::Texture>) -> Result<(), wgpu::SurfaceError> {
        if textures.len() != self.render_config.displays_indices.len()
            || textures.len() != self.render_config.displays_vertices.len()
        {
            panic!("Number of textures doesn't match number of displays provided.")
        }

        let vertex_buffers = self
            .render_config
            .displays_vertices
            .iter()
            .map(|vertices| {
                wgpu::util::DeviceExt::create_buffer_init(
                    &*self.device,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Renderer Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                )
            })
            .collect::<Vec<_>>();

        let index_buffers = self
            .render_config
            .displays_indices
            .iter()
            .map(|indices| {
                wgpu::util::DeviceExt::create_buffer_init(
                    &*self.device,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Renderer Index Buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                )
            })
            .collect::<Vec<_>>();

        let texture_bind_groups = textures
            .into_iter()
            .map(|texture| {
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Renderer Display Texture Bind Group"),
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.texture_sampler),
                        },
                    ],
                })
            })
            .collect::<Vec<_>>();

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Renderer Command Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Renderer Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            for i in 0..texture_bind_groups.len() {
                render_pass.set_bind_group(0, &texture_bind_groups[i], &[]);
                render_pass.set_vertex_buffer(0, vertex_buffers[i].slice(..));
                render_pass.set_index_buffer(index_buffers[i].slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(
                    0..self.render_config.displays_indices[i].len() as u32,
                    0,
                    0..1,
                );
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
