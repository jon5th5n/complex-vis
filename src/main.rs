use std::cell::RefCell;
use std::sync::Arc;
use std::time::Instant;

mod gpu_renderer;
use gpu_renderer::*;

mod gpu_canvas;
use gpu_canvas::*;

use wgpu::util::DeviceExt;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

mod graph2d;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

fn main() {
    env_logger::init();
    println!("hello world");

    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().expect("Failed to create Event Loop.");
    let window = WindowBuilder::new()
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();

    let render_config = RenderConfig::new_rects(&[RectDescriptor {
        upper_left: (-1.0, 1.0),
        lower_rigth: (1.0, -1.0),
    }]);

    let mut renderer = GPURenderer::new(window, render_config).await.unwrap();

    let mut canvas = GPUCanvas::new(WIDTH, HEIGHT, renderer.device_arc(), renderer.queue_arc());

    let graphing_desc = GraphingCanvasDesc {
        size: [WIDTH, HEIGHT],
        margin: [25, 25],
        range_start: [-10.0, -10.0],
        range_end: [10.0, 10.0],
    };

    let graphing_desc_buffer =
        canvas
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&graphing_desc),
                usage: wgpu::BufferUsages::UNIFORM,
            });

    let graphing_desc_layout =
        canvas
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

    let graphing_desc_bind_group = canvas
        .device()
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &graphing_desc_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: graphing_desc_buffer.as_entire_binding(),
            }],
        });

    canvas.set_additional(graphing_desc_layout, graphing_desc_bind_group);

    let draw_clear = GPUDrawClear::new_arc([1.0, 1.0, 1.0, 1.0]);
    let draw_function = GPUDrawFunction::new_arc([0.0, 0.0, 0.0, 1.0]);

    canvas.load_drawing_ops(vec![draw_clear.clone(), draw_function.clone()]);

    let mut timer = Instant::now();

    let mut counter = 0u8;

    event_loop
        .run(move |event, control_flow| match event {
            Event::AboutToWait => {
                renderer.window().request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window().id() => {
                match event {
                    WindowEvent::RedrawRequested => {
                        counter += 1;
                        let g = counter as f32 / u8::MAX as f32;

                        draw_function.borrow_mut().set_color([g, 0.0, 1.0 - g, 1.0]);
                        canvas.render();

                        match renderer.render(vec![canvas.texture()]) {
                            Ok(_) => {}
                            // Reconfigure the surface if lostdraw_texture_vars.offset
                            Err(wgpu::SurfaceError::Lost) => {
                                renderer.window_resize(renderer.window_size())
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }

                        let now = Instant::now();
                        let delta = now - timer;
                        println!("{:?}", delta.as_millis());
                        timer = now;
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            winit::event::KeyEvent {
                                logical_key:
                                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        renderer.window_resize(*physical_size);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .expect("Event Loop Error.");
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GraphingCanvasDesc {
    size: [u32; 2],

    margin: [u32; 2],

    range_start: [f32; 2],
    range_end: [f32; 2],
}

pub struct GPUDrawFunction {
    color: [f32; 4],
    buffer: Option<wgpu::Buffer>,
}

impl GPUDrawFunction {
    pub fn new(color: [f32; 4]) -> Self {
        Self {
            color,
            buffer: None,
        }
    }

    pub fn new_arc(color: [f32; 4]) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(Self::new(color)))
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }
}

impl GPUDrawOp for GPUDrawFunction {}

impl GPUDrawOpStatic for GPUDrawFunction {
    fn shader(&self) -> &'static str {
        include_str!("draw_function.wgsl")
    }

    fn bind_group_layout_descriptor(&self) -> wgpu::BindGroupLayoutDescriptor {
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }
}

impl GPUDrawOpDynamic for GPUDrawFunction {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let data = bytemuck::bytes_of(&self.color);

        match &self.buffer {
            Some(buffer) => queue.write_buffer(buffer, 0, data),
            None => {
                self.buffer = Some(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: data,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }),
                )
            }
        }
    }

    fn create_bind_group(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        if self.buffer.is_none() {
            self.update(device, queue);
        }

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_ref().unwrap().as_entire_binding(),
            }],
        })
    }
}
