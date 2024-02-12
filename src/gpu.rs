use std::fmt::format;

use bytemuck;
use drawing_stuff::color;
use wgpu::util::DeviceExt;

pub trait SDF {
    /// Returns the name of the signed distance function.
    fn sdf_name(&self) -> String;

    /// Returns the signed distance function declaration.
    fn sdf_dec(&self) -> String;

    /// Returns the signed distance function caller.
    fn sdf_cal(&self) -> String;

    // Returns the color.
    fn color(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct Circle2D {
    pub center: (f32, f32),
    pub radius: f32,

    pub color: u32,
}

impl SDF for Circle2D {
    fn sdf_name(&self) -> String {
        "sdf_circle".to_string()
    }

    fn sdf_dec(&self) -> String {
        r#"
fn sdf_circle(p: vec2f, center: vec2f, radius: f32) -> f32 {
    return length(center - p) - radius;
}
        "#
        .to_string()
    }

    fn sdf_cal(&self) -> String {
        format!(
            "sdf_circle(p, vec2f({:?}, {:?}), {:?})",
            self.center.0, self.center.1, self.radius
        )
    }

    fn color(&self) -> u32 {
        self.color
    }
}

const SDF2_BASE_SHADER: &'static str = r#"
struct Globals {
    pixels_width: u32,
    pixels_height: u32,
}

@group(0)
@binding(0)
var<uniform> globals: Globals;

@group(0)
@binding(1)
var<storage, read_write> pixels: array<u32>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let p = vec2f(f32(global_id.x), f32(global_id.y));

    let min_sdf = $sdf_expr$

    if min_sdf[3] <= 0.0 {
        pixels[global_id.y * globals.pixels_width + global_id.x] = rgb_to_u32(min_sdf.xyz);
    } else {
        pixels[global_id.y * globals.pixels_width + global_id.x] = 0xFFFFFFFFu;
    }
}

fn rgb_to_u32(rgb: vec3f) -> u32 {
    let r = u32(rgb.x * 255);
    let g = u32(rgb.y * 255);
    let b = u32(rgb.z * 255);

    return (r << 16) | (g << 8) | b;
}

fn sdf_union(one: vec4f, two: vec4f) -> vec4f {
    if (one[3] < two[3]) {
        return one;
    };
    return two;
}
"#;

pub struct SDF2Constructor {
    width: u32,
    height: u32,

    sdf_buf: Vec<Box<dyn SDF>>,

    base_shader: String,
    compute_runner: SDFCompute,
}

impl SDF2Constructor {
    pub async fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            sdf_buf: Vec::new(),
            base_shader: SDF2_BASE_SHADER.to_string(),
            compute_runner: SDFCompute::new().await,
        }
    }

    pub fn add_sdf(&mut self, sdf: Box<dyn SDF>) {
        self.sdf_buf.push(sdf);
    }

    pub fn compile(&mut self) {
        let mut shader = self.base_shader.clone();

        let mut included_sdfs: Vec<String> = Vec::new();

        let mut sdf_expr = "$replace$;".to_string();
        for i in 0..self.sdf_buf.len() {
            let sdf = &self.sdf_buf[i];

            if !included_sdfs.contains(&sdf.sdf_name()) {
                shader = format!("{}\n\n{}", shader, sdf.sdf_dec());
                included_sdfs.push(sdf.sdf_name());
            }

            let c = sdf.color();
            let r = ((c >> 16) & 0x00FF0000) as f32 / 255.0;
            let g = ((c >> 8) & 0x0000FF00) as f32 / 255.0;
            let b = ((c >> 0) & 0x000000FF) as f32 / 255.0;

            let vec4 = format!("vec4f({:?}, {:?}, {:?}, {})", r, g, b, sdf.sdf_cal(),);

            if i < self.sdf_buf.len() - 1 {
                let expr = format!("sdf_union({}, $replace$)", vec4);
                sdf_expr = sdf_expr.replace("$replace$", &expr);
                continue;
            }

            sdf_expr = sdf_expr.replace("$replace$", &vec4);
        }

        let shader = shader.replace("$sdf_expr$", &sdf_expr);

        println!("BA");

        self.compute_runner.set_shader(&shader);

        println!("BB");
    }

    pub async fn run(&self) -> Vec<u32> {
        self.compute_runner
            .run_shader(self.width, self.height)
            .await
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    pixels_width: u32,
    pixels_height: u32,
}

pub struct SDFCompute {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    features: wgpu::Features,
    device: wgpu::Device,
    queue: wgpu::Queue,

    cs_module: wgpu::ShaderModule,
}
impl SDFCompute {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });
        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        let features = adapter.features();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Default::default()),
        });

        SDFCompute {
            instance,
            adapter,
            features,
            device,
            queue,
            cs_module,
        }
    }

    pub fn set_shader(&mut self, shader_code: &str) {
        self.cs_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(shader_code.into()),
            });
    }

    pub async fn run_shader(&self, width: u32, height: u32) -> Vec<u32> {
        let globals = Globals {
            pixels_width: width,
            pixels_height: height,
        };
        let globals = bytemuck::bytes_of(&globals);
        let globals_uni = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: globals,
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group_layout_globals_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let bind_group_globals_entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: globals_uni.as_entire_binding(),
        };

        let pixels_len = width as u64 * height as u64 * 4;

        let pixel_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: pixels_len,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group_layout_pixel_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let bind_group_pixels_entry = wgpu::BindGroupEntry {
            binding: 1,
            resource: pixel_buf.as_entire_binding(),
        };

        let return_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: pixels_len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        bind_group_layout_globals_entry,
                        bind_group_layout_pixel_entry,
                    ],
                });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[bind_group_globals_entry, bind_group_pixels_entry],
        });

        let compute_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&compute_pipeline_layout),
                module: &self.cs_module,
                entry_point: "main",
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(width, height, 1);
        }
        encoder.copy_buffer_to_buffer(&pixel_buf, 0, &return_buf, 0, pixels_len);
        self.queue.submit(Some(encoder.finish()));

        let buf_slice = return_buf.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buf_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        // Assume that both buffers become available at the same time. A more careful
        // approach would be to wait for both notifications to be sent.
        self.device.poll(wgpu::Maintain::Wait);
        let _ = receiver.receive().await;
        let data_raw = &*buf_slice.get_mapped_range();

        bytemuck::cast_slice(data_raw).to_vec()
    }
}
