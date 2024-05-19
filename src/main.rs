mod gpuview;
use gpuview::*;
use wgpu_text::glyph_brush::{Section, Text};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use anyhow::Context;
use pollster::FutureExt;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct ExampleShaderDesc {
    offset: [f32; 3],

    buffer: Option<wgpu::Buffer>,

    is_initialized: bool,
    offset_changed: bool,
}

impl ExampleShaderDesc {
    pub fn new(offset: [f32; 3]) -> Self {
        Self {
            offset,
            buffer: None,
            is_initialized: false,
            offset_changed: false,
        }
    }

    pub fn set_offset(&mut self, offset: [f32; 3]) {
        self.offset = offset;
        self.offset_changed = true;
    }

    pub fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(self))
    }
}

impl ShaderDescriptor for ExampleShaderDesc {
    fn initialize(&mut self, device: &wgpu::Device) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Descriptor Buffer"),
            contents: bytemuck::bytes_of(&self.offset),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        self.buffer = Some(buffer);
        self.is_initialized = true;
    }

    fn update_buffers(&self, queue: &wgpu::Queue) {
        if self.offset_changed {
            queue.write_buffer(
                self.buffer.as_ref().unwrap(),
                0,
                bytemuck::bytes_of(&self.offset),
            )
        }
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
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
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
                resource: self.buffer.as_ref().unwrap().as_entire_binding(),
            }],
        });

        (bind_group, bind_group_layout)
    }
}

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
    width: u32,
    height: u32,

    x_margin: f32,
    y_margin: f32,

    x_range: Range<f32>,
    y_range: Range<f32>,

    shader_descriptor: Arc<RefCell<GPUCanvas2DShaderDescriptor>>,
    view: Arc<RefCell<GPUView<'a>>>,
}

impl GPUCanvas2D<'_> {
    pub fn new(width: u32, height: u32, x_margin: f32, y_margin: f32) -> Self {
        let shader_descriptor = GPUCanvas2DShaderDescriptor::new().into_arc_ref_cell();

        Self {
            width,
            height,
            x_margin,
            y_margin,
            x_range: -1.0..1.0,
            y_range: -1.0..1.0,
            shader_descriptor: shader_descriptor.clone(),
            view: GPUView::new(width, height, shader_descriptor, todo!()).into_arc_ref_cell(),
        }
    }
}

struct App<'a> {
    window: Option<Arc<Window>>,

    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,

    multiview: GPUMultiView<'a>,

    prev_t: std::time::Instant,
    delta_t: std::time::Duration,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            window: None,
            device: None,
            queue: None,
            multiview: GPUMultiView::new(),
            prev_t: std::time::Instant::now(),
            delta_t: std::time::Duration::ZERO,
        }
    }

    pub async fn initialize(&mut self, window: Window) -> anyhow::Result<()> {
        let window = Arc::new(window);

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

        let surface_format = wgpu::TextureFormat::Bgra8Unorm;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: surface_format.required_features()
                        | wgpu::Features::BGRA8UNORM_STORAGE
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        | wgpu::Features::TEXTURE_BINDING_ARRAY
                        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                        | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
                        | wgpu::Features::POLYGON_MODE_LINE
                        | wgpu::Features::POLYGON_MODE_POINT
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

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        self.multiview.initialize(surface, surface_config, &device);

        self.window = Some(window);
        self.device = Some(Arc::new(device));
        self.queue = Some(Arc::new(queue));

        Ok(())
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.initialize(
            event_loop
                .create_window(Window::default_attributes().with_transparent(true))
                .unwrap(),
        )
        .block_on()
        .unwrap();

        let shader_desc = ExampleShaderDesc::new([-0.5, 0.5, 0.0]).into_arc_ref_cell();

        let width = self.multiview.width().unwrap();
        let height = self.multiview.height().unwrap();

        let multi_section = Arc::new(RefCell::new(
            Section::builder()
                .add_text(
                    Text::new("Multiview")
                        .with_scale(96.0)
                        .with_color([0.0, 1.0, 0.0, 1.0]),
                )
                .with_screen_position((10.0, height as f32 / 1.5))
                .to_owned(),
        ));

        let multi_text = TextPrimitive::new(
            include_bytes!("../fonts/JetBrainsMono-Italic.ttf"),
            vec![multi_section],
        );

        self.multiview.set_text_primitives(vec![multi_text]);

        let mut render_verts = Vec::new();
        vertices_add_line(
            &mut render_verts,
            [-1.0, 1.0],
            [1.0, 0.0],
            0.05,
            [0.0, 0.0, 1.0, 1.0],
        );

        let section11 = Arc::new(RefCell::new(
            Section::builder()
                .add_text(
                    Text::new("Hello, world.\nHere is section1!")
                        .with_scale(26.0)
                        .with_color([0.8, 0.2, 0.5, 1.0]),
                )
                .with_screen_position((width as f32 / 2.0, height as f32 / 2.0))
                .to_owned(),
        ));

        let section12 = Arc::new(RefCell::new(
            Section::builder()
                .add_text(
                    Text::new("Bye, world.\nHere is section2!")
                        .with_scale(50.0)
                        .with_color([0.1, 0.7, 1.0, 1.0]),
                )
                .with_screen_position((width as f32 / 10.0, height as f32 / 1.5))
                .to_owned(),
        ));

        let text1 = TextPrimitive::new(
            include_bytes!("../fonts/JetBrainsMono-Regular.ttf"),
            vec![section11, section12],
        );

        let section21 = Arc::new(RefCell::new(
            Section::builder()
                .add_text(
                    Text::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
                        .with_scale(10.0)
                        .with_color([0.0, 0.0, 0.0, 1.0]),
                )
                .with_screen_position((width as f32 / 20.0, 10.0))
                .to_owned(),
        ));

        let section22 = Arc::new(RefCell::new(
            Section::builder()
                .add_text(
                    Text::new("123456789")
                        .with_scale(14.0)
                        .with_color([0.0, 0.0, 0.0, 1.0]),
                )
                .with_screen_position((width as f32 / 20.0, 30.0))
                .to_owned(),
        ));

        let text2 = TextPrimitive::new(
            include_bytes!("../fonts/JetBrainsMono-Bold.ttf"),
            vec![section21, section22],
        );

        let mut view1 =
            GPUView::new_rect_frame(width, height, shader_desc.clone(), [-1.0, 1.0], [0.0, -0.0]);

        view1.set_clear_color(wgpu::Color::WHITE);

        view1.set_render_vertices(render_verts);

        view1.set_text_primitives(vec![text1, text2]);

        let view1 = view1.into_arc_ref_cell();

        let mut view2 =
            GPUView::new_rect_frame(width, height, shader_desc.clone(), [-0.0, 0.0], [1.0, -1.0]);

        view2.set_clear_color(wgpu::Color::RED);

        let view2 = view2.into_arc_ref_cell();

        self.multiview.set_render_views(vec![view1, view2]);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                // println!("{:?}", WindowEvent::Resized(new_size));
                self.multiview.resize(
                    new_size.width,
                    new_size.height,
                    self.device.as_ref().unwrap(),
                );
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                self.delta_t = now - self.prev_t;
                self.prev_t = now;

                println!("{:?}", self.delta_t.as_millis());

                self.multiview
                    .render(self.device.as_ref().unwrap(), self.queue.as_ref().unwrap());

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                winit::keyboard::PhysicalKey::Code(key_code) => {
                    if key_code == winit::keyboard::KeyCode::Escape {
                        event_loop.exit();
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();
    println!("Hello, world!");

    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();

    let _ = event_loop.run_app(&mut app);
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
