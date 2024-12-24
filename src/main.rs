mod math;
use math::*;

mod decimal_math;
use decimal_math::*;

mod color;
use color::*;

mod gpuview;
use gpuview::{Font, *};

mod graph;
use graph::*;

mod gpucanvas_2d;
use gpucanvas_2d::*;

use wgpu_text::glyph_brush::ab_glyph::{FontArc, PxScale};
use wgpu_text::glyph_brush::{
    Extra, HorizontalAlign, Layout, OwnedSection, OwnedText, Section, Text, VerticalAlign,
};
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
use winit::event::{self, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Debug, Default)]
struct GraphParam {
    a: f64,
}

struct App<'a> {
    window: Option<Arc<Window>>,

    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,

    multiview: GPUMultiView<'a>,
    canvas: GPUCanvas2D<GraphParam>,

    mouse_pos: PhysicalPosition<f64>,
    mouse_delta: PhysicalPosition<f64>,
    mouse_left: bool,

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
            canvas: GPUCanvas2D::new(GPUViewFrame::Whole.with_margin((0.1, 0.1))),
            mouse_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            mouse_delta: PhysicalPosition { x: 0.0, y: 0.0 },
            mouse_left: false,
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
        self.multiview.set_clear_color(wgpu::Color::WHITE);

        {
            let canvas_style = self.canvas.style_get_mut();

            // canvas_style.text = None;
        }

        let square = FunctionGraph {
            function: |x: f64, p: &GraphParam| (x - p.a).powi(2),
            style: GraphStyle {
                color: RGBA::new(131, 39, 196, 255),
                thickness: Thickness::MEDIUM,
            },
        };

        let exp = FunctionGraph {
            function: |x: f64, p: &GraphParam| (x * p.a).exp(),
            style: GraphStyle {
                color: RGBA::new(39, 187, 204, 255),
                thickness: Thickness::EXTRABOLD,
            },
        };

        let cos = FunctionGraph {
            function: |x: f64, p: &GraphParam| x.cos() - p.a,
            style: GraphStyle {
                color: RGBA::new(230, 178, 57, 255),
                thickness: Thickness::THIN,
            },
        };

        self.canvas.add_function_graph(square);
        self.canvas.add_function_graph(exp);
        self.canvas.add_function_graph(cos);

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

        self.canvas.set_clear_color(RGBA::WHITE);

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

                let num_vertices = self
                    .canvas
                    .get_view()
                    .as_ref()
                    .borrow()
                    .get_render_vertices_len();

                println!(
                    "{}ms with {} vertices",
                    self.delta_t.as_micros() as f32 / 1000.0,
                    num_vertices
                );

                self.canvas.display();

                let x =
                    (self.mouse_pos.x as f32 / self.multiview.width().unwrap() as f32) * 2.0 - 1.0;
                let y = -((self.mouse_pos.y as f32 / self.multiview.height().unwrap() as f32)
                    * 2.0
                    - 1.0);

                let view_coords = self.multiview.get_view_coords_behind((x, y));

                // println!(
                //     "{:?}, {:?}, {:?}, {:?}",
                //     self.canvas.x_range(),
                //     self.canvas.x_range_len(),
                //     self.canvas.y_range(),
                //     self.canvas.y_range_len()
                // );

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

                    KeyCode::ArrowUp => self.canvas.parameter_get_mut().a += 0.1,
                    KeyCode::ArrowDown => self.canvas.parameter_get_mut().a -= 0.1,
                    _ => {}
                },
                _ => (),
            },
            WindowEvent::MouseInput { button, state, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => self.mouse_left = true,
                (MouseButton::Left, ElementState::Released) => self.mouse_left = false,
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_delta = PhysicalPosition {
                    x: position.x - self.mouse_pos.x,
                    y: position.y - self.mouse_pos.y,
                };

                if self.mouse_left {
                    let x =
                        (position.x as f32 / self.multiview.width().unwrap() as f32) * 2.0 - 1.0;
                    let y = -((position.y as f32 / self.multiview.height().unwrap() as f32) * 2.0
                        - 1.0);

                    let prev_x = (self.mouse_pos.x as f32 / self.multiview.width().unwrap() as f32)
                        * 2.0
                        - 1.0;
                    let prev_y = -((self.mouse_pos.y as f32
                        / self.multiview.height().unwrap() as f32)
                        * 2.0
                        - 1.0);

                    let view_pos = self.multiview.get_view_coords_behind((x, y));

                    let prev_view_pos = self.multiview.get_view_coords_behind((prev_x, prev_y));

                    match (view_pos, prev_view_pos) {
                        (Some(view_pos), Some(prev_view_pos)) => {
                            if view_pos.view_index != prev_view_pos.view_index {
                                return;
                            }

                            let dx = view_pos.coordinates.0 - prev_view_pos.coordinates.0;
                            let dy = view_pos.coordinates.1 - prev_view_pos.coordinates.1;

                            let x_rng_len = self.canvas.x_range_len();
                            let y_rng_len = self.canvas.y_range_len();

                            self.canvas.offset_range((
                                -dx as f64 * x_rng_len * 0.5,
                                -dy as f64 * y_rng_len * 0.5,
                            ));
                        }
                        _ => {}
                    }
                }

                self.mouse_pos = position;
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                event::MouseScrollDelta::LineDelta(x, y) => {
                    let scale = 1.0 - (y * 0.05);
                    self.canvas.scale_range((scale as f64, scale as f64))
                }
                event::MouseScrollDelta::PixelDelta(amt) => (),
            },
            _ => (),
        }
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "0");

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
