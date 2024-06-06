mod gpuview;
use gpuview::*;

mod gpucanvas_2d;
use gpucanvas_2d::*;

use wgpu_text::glyph_brush::{Section, Text};
use winit::dpi::PhysicalPosition;
use winit::keyboard::{KeyCode, PhysicalKey};

use core::cell::RefCell;
use std::borrow::{Borrow, BorrowMut};
use std::ops::Range;
use std::sync::Arc;

use anyhow::Context;
use pollster::FutureExt;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

// struct ExampleShaderDesc {
//     offset: [f32; 3],

//     buffer: Option<wgpu::Buffer>,

//     is_initialized: bool,
//     offset_changed: bool,
// }

// impl ExampleShaderDesc {
//     pub fn new(offset: [f32; 3]) -> Self {
//         Self {
//             offset,
//             buffer: None,
//             is_initialized: false,
//             offset_changed: false,
//         }
//     }

//     pub fn set_offset(&mut self, offset: [f32; 3]) {
//         self.offset = offset;
//         self.offset_changed = true;
//     }

//     pub fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
//         Arc::new(RefCell::new(self))
//     }
// }

// impl ShaderDescriptor for ExampleShaderDesc {
//     fn initialize(&mut self, device: &wgpu::Device) {
//         let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Shader Descriptor Buffer"),
//             contents: bytemuck::bytes_of(&self.offset),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         });

//         self.buffer = Some(buffer);
//         self.is_initialized = true;
//     }

//     fn update_buffers(&self, queue: &wgpu::Queue) {
//         if self.offset_changed {
//             queue.write_buffer(
//                 self.buffer.as_ref().unwrap(),
//                 0,
//                 bytemuck::bytes_of(&self.offset),
//             )
//         }
//     }

//     fn shader_source(&self) -> wgpu::ShaderSource {
//         wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
//     }

//     fn bind_group_and_layout(
//         &self,
//         device: &wgpu::Device,
//     ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
//         let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: Some("Shader Descripot Bind Group Layout"),
//             entries: &[wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             }],
//         });

//         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//             label: Some("Shader Descriptor Bind Group"),
//             layout: &bind_group_layout,
//             entries: &[wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: self.buffer.as_ref().unwrap().as_entire_binding(),
//             }],
//         });

//         (bind_group, bind_group_layout)
//     }
// }

struct App<'a> {
    window: Option<Arc<Window>>,

    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,

    multiview: GPUMultiView<'a>,
    canvas: GPUCanvas2D<'a>,

    mouse_pos: PhysicalPosition<f64>,

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
            canvas: GPUCanvas2D::new(GPUViewFrame::Whole),
            mouse_pos: PhysicalPosition { x: 0.0, y: 0.0 },
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

        self.canvas.set_clear_color(wgpu::Color::WHITE);
        self.canvas.add_function(|x| x.powi(2));

        let canvas_view = self.canvas.get_view();

        self.multiview.set_render_views(vec![canvas_view]);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                // println!("{:?}", WindowEvent::Resized(new_size));
                let _ = self.multiview.resize(
                    new_size.width,
                    new_size.height,
                    self.device.as_ref().unwrap(),
                );
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                self.delta_t = now - self.prev_t;
                self.prev_t = now;

                // println!("{:?}", self.delta_t.as_millis());

                self.canvas
                    .get_view()
                    .as_ref()
                    .borrow_mut()
                    .clear_render_vertices();
                self.canvas.add_function(|x| (x.powi(2)));

                let x = self.mouse_pos.x as f32 / self.multiview.width().unwrap() as f32;
                let y = self.mouse_pos.y as f32 / self.multiview.height().unwrap() as f32;

                let _ = self
                    .multiview
                    .render(self.device.as_ref().unwrap(), self.queue.as_ref().unwrap());

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(key_code) => match key_code {
                    KeyCode::Escape => {
                        event_loop.exit();
                    }
                    KeyCode::KeyD => self.canvas.offset_range((0.2, 0.0)),
                    KeyCode::KeyA => self.canvas.offset_range((-0.2, 0.0)),
                    KeyCode::KeyW => self.canvas.offset_range((0.0, 0.2)),
                    KeyCode::KeyS => self.canvas.offset_range((0.0, -0.2)),
                    _ => {}
                },
                _ => (),
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = position;
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    self.canvas.scale_range(1.0 - (y * 0.1))
                }
                winit::event::MouseScrollDelta::PixelDelta(amt) => (),
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
