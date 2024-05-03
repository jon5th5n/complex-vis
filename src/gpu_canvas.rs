use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    sync::Arc,
};
use wgpu::{hal::Queue, util::DeviceExt};

pub trait GPUDrawOp: GPUDrawOpStatic + GPUDrawOpDynamic + Any {}

pub trait GPUDrawOpStatic {
    fn shader(&self) -> &'static str;
    fn bind_group_layout_descriptor(&self) -> wgpu::BindGroupLayoutDescriptor;
}

pub trait GPUDrawOpDynamic {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn create_bind_group(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup;
}

pub struct GPUDrawClear {
    color: [f32; 4],
    buffer: Option<wgpu::Buffer>,
}

impl GPUDrawClear {
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

impl GPUDrawOp for GPUDrawClear {}

impl GPUDrawOpStatic for GPUDrawClear {
    fn shader(&self) -> &'static str {
        r#"
            @group(0) @binding(0)
            var texture: texture_storage_2d<bgra8unorm, read_write>;

            @group(2) @binding(0)
            var<uniform> color: vec4<f32>;

            @compute
            @workgroup_size(1)
            fn draw(@builtin(workgroup_id) id: vec3<u32>, @builtin(num_workgroups) size: vec3<u32>) {
                textureStore(texture, id.xy, color);
            }
        "#
    }

    fn bind_group_layout_descriptor(&self) -> wgpu::BindGroupLayoutDescriptor {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Draw Clear Bind Group Layout"),
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

impl GPUDrawOpDynamic for GPUDrawClear {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let data = bytemuck::bytes_of(&self.color);

        match &self.buffer {
            Some(buffer) => queue.write_buffer(buffer, 0, data),
            None => {
                self.buffer = Some(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Draw Clear Buffer"),
                        contents: data,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }),
                )
            }
        };
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

        let buffer = self.buffer.as_ref().unwrap();

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Draw Clear Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}

pub struct GPUDrawTexture {
    width: u32,
    height: u32,

    data: Vec<u8>,
    texture: Option<wgpu::Texture>,

    offset: [u32; 2],
    buffer: Option<wgpu::Buffer>,
}

impl GPUDrawTexture {
    pub fn new(width: u32, height: u32, data: Vec<u8>, offset: [u32; 2]) -> Self {
        if data.len() as u32 != width * height * 4 {
            panic!(
                "Data is not of the right length for given size. Needs to be width * height * 4."
            )
        }

        Self {
            width,
            height,
            data,
            texture: None,
            offset,
            buffer: None,
        }
    }

    pub fn new_arc(width: u32, height: u32, data: Vec<u8>, offset: [u32; 2]) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(Self::new(width, height, data, offset)))
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        if data.len() as u32 != self.width * self.height * 4 {
            panic!(
                "Data is not of the right length for given size. Needs to be width * height * 4."
            )
        }

        self.data = data;
    }

    pub fn set_offset(&mut self, offset: [u32; 2]) {
        self.offset = offset;
    }
}

impl GPUDrawOp for GPUDrawTexture {}

impl GPUDrawOpStatic for GPUDrawTexture {
    fn shader(&self) -> &'static str {
        r#"
            @group(0) @binding(0)
            var texture: texture_storage_2d<bgra8unorm, read_write>;

            @group(2) @binding(0)
            var<uniform> offset: vec2<u32>;

            @group(2) @binding(1)
            var draw_texture_texture: texture_storage_2d<bgra8unorm, read>;

            @compute
            @workgroup_size(1)
            fn draw(@builtin(workgroup_id) id: vec3<u32>, @builtin(num_workgroups) size: vec3<u32>) {
                if (id.x < offset.x || id.y < offset.y) {
                    return;
                }
                let draw_pos = id.xy - offset;

                let draw_size = textureDimensions(draw_texture_texture);
                if (draw_pos.x >= draw_size.x || draw_pos.y >= draw_size.y) {
                    return;
                }
                let color = textureLoad(draw_texture_texture, draw_pos);

                textureStore(texture, id.xy, color);
            }
        "#
    }

    fn bind_group_layout_descriptor(&self) -> wgpu::BindGroupLayoutDescriptor {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Draw Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        }
    }
}

impl GPUDrawOpDynamic for GPUDrawTexture {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let texture_data = self.data.as_slice();
        match &self.texture {
            Some(texture) => queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                texture_data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.width * 4),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
            ),
            None => {
                self.texture = Some(device.create_texture_with_data(
                    queue,
                    &wgpu::TextureDescriptor {
                        label: Some("Draw Texture Texture"),
                        size: wgpu::Extent3d {
                            width: self.width,
                            height: self.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        usage: wgpu::TextureUsages::STORAGE_BINDING
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                    Default::default(),
                    texture_data,
                ))
            }
        }

        let offset_data = bytemuck::bytes_of(&self.offset);
        match &self.buffer {
            Some(buffer) => queue.write_buffer(buffer, 0, offset_data),
            None => {
                self.buffer = Some(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Draw Texture Buffer"),
                        contents: offset_data,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }),
                );
            }
        };
    }

    fn create_bind_group(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        if self.texture.is_none() || self.buffer.is_none() {
            self.update(device, queue);
        }

        let buffer = self.buffer.as_ref().unwrap();
        let texture = self.texture.as_ref().unwrap();

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Draw Texture Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        })
    }
}

struct GPUDrawOpStaticContent {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

struct GPUDrawOpDynamicContent {
    op: Arc<RefCell<dyn GPUDrawOp>>,
    bind_group: wgpu::BindGroup,
}

pub struct GPUCanvas {
    // GPU Handle
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // GPU Canvas
    width: u32,
    height: u32,

    texture: wgpu::Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,

    additional_bind_group_layout: wgpu::BindGroupLayout,
    additional_bind_group: wgpu::BindGroup,

    loaded_ops_static: HashMap<TypeId, GPUDrawOpStaticContent>,
    drawing_buffer: Vec<GPUDrawOpDynamicContent>,

    premultiply_pipeline: wgpu::ComputePipeline,
}

impl GPUCanvas {
    const PREMULTIPLY_SHADER: &'static str = r#"
        @group(0) @binding(0)
        var texture: texture_storage_2d<bgra8unorm, read_write>;

        @compute
        @workgroup_size(1)
        fn premultiply(@builtin(workgroup_id) id: vec3<u32>, @builtin(num_workgroups) size: vec3<u32>) {
            let color = textureLoad(texture, id.xy);
            textureStore(texture, id.xy, vec4<f32>(color.xyz * color.w, color.w));
        }
    "#;

    pub fn new(
        width: u32,
        height: u32,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Canvas Texture Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        let additional_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Canvas Additional Bind Group Layout"),
                entries: &[],
            });

        let additional_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Additional Bind Group"),
            layout: &additional_bind_group_layout,
            entries: &[],
        });

        let loaded_ops_static = HashMap::new();
        let drawing_buffer = Vec::new();

        let premultiply_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Canvas Premultiply Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Self::PREMULTIPLY_SHADER.into()),
        });

        let premultiply_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Canvas Premultiply Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let premultiply_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Canvas Premultiply Pipeline"),
                layout: Some(&premultiply_pipeline_layout),
                module: &premultiply_module,
                entry_point: "premultiply",
            });

        Self {
            device,
            queue,
            width,
            height,
            texture,
            texture_bind_group_layout,
            texture_bind_group,
            additional_bind_group_layout,
            additional_bind_group,
            loaded_ops_static,
            drawing_buffer,
            premultiply_pipeline,
        }
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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;

        self.texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        self.texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &self
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });
    }

    pub fn set_additional(
        &mut self,
        bind_group_layout: wgpu::BindGroupLayout,
        bind_group: wgpu::BindGroup,
    ) {
        self.additional_bind_group_layout = bind_group_layout;
        self.additional_bind_group = bind_group;
    }

    pub fn reset_additional(&mut self) {
        self.additional_bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Canvas Additional Bind Group Layout"),
                    entries: &[],
                });

        self.additional_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Additional Bind Group"),
            layout: &self.additional_bind_group_layout,
            entries: &[],
        });
    }

    pub fn load_drawing_ops(&mut self, drawing_ops: Vec<Arc<RefCell<dyn GPUDrawOp>>>) {
        self.drawing_buffer.clear();

        for op in drawing_ops {
            let id = (&*op.borrow()).type_id();

            let static_content = match self.loaded_ops_static.get(&id) {
                Some(content) => content,
                None => {
                    let shader_module =
                        self.device
                            .create_shader_module(wgpu::ShaderModuleDescriptor {
                                label: Some("Canvas Drawing Operation Shader Module"),
                                source: wgpu::ShaderSource::Wgsl(op.borrow().shader().into()),
                            });

                    let bind_group_layout = self
                        .device
                        .create_bind_group_layout(&op.borrow().bind_group_layout_descriptor());

                    let pipeline_layout =
                        self.device
                            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                label: Some("Canvas Drawing Operation Pipeline Layout"),
                                bind_group_layouts: &[
                                    &self.texture_bind_group_layout,
                                    &self.additional_bind_group_layout,
                                    &bind_group_layout,
                                ],
                                push_constant_ranges: &[],
                            });

                    let pipeline =
                        self.device
                            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                label: Some("Canvas Drawing Operation Pipeline"),
                                layout: Some(&pipeline_layout),
                                module: &shader_module,
                                entry_point: "draw",
                            });

                    let content = GPUDrawOpStaticContent {
                        bind_group_layout,
                        pipeline,
                    };

                    self.loaded_ops_static.insert(id, content);
                    self.loaded_ops_static.get(&id).unwrap()
                }
            };

            let bind_group = op.borrow_mut().create_bind_group(
                &self.device,
                &self.queue,
                &static_content.bind_group_layout,
            );

            let dynamic_content = GPUDrawOpDynamicContent {
                op: op.clone(),
                bind_group,
            };

            self.drawing_buffer.push(dynamic_content);
        }
    }

    pub fn render(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Canvas Command Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Canvas Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.additional_bind_group, &[]);

            for dynamic_content in &self.drawing_buffer {
                dynamic_content
                    .op
                    .borrow_mut()
                    .update(&self.device, &self.queue);

                let id = (&*dynamic_content.op.borrow()).type_id();

                let static_content = self
                    .loaded_ops_static
                    .get(&id)
                    .expect("Used drawing operation wasn't loaded correctly prior to use.");

                let pipeline = &static_content.pipeline;
                let bind_group = &dynamic_content.bind_group;

                compute_pass.set_bind_group(2, bind_group, &[]);
                compute_pass.set_pipeline(pipeline);
                compute_pass.dispatch_workgroups(self.width, self.height, 1);
            }

            compute_pass.set_pipeline(&self.premultiply_pipeline);
            compute_pass.dispatch_workgroups(self.width, self.height, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
