use std::{fs, marker::PhantomData, time::Instant};

use pollster::FutureExt as _;
use wgpu::util::DeviceExt;

use super::{NoUserData, TuiShaderBackend};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderInput {
    // struct field order matters
    time: f32,
    padding: f32,
    resolution: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct WgpuBackend<T>
where
    T: Copy + Clone + Default + bytemuck::Pod + bytemuck::Zeroable,
{
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    output_buffer: wgpu::Buffer,
    shader_input_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    creation_time: Instant,
    width: u16,
    height: u16,
    _user_data: PhantomData<T>,
}

impl<T> WgpuBackend<T>
where
    T: Copy + Clone + Default + bytemuck::Pod + bytemuck::Zeroable,
{
    pub fn new(path_to_fragment_shader: &str, entry_point: &str) -> Self {
        Self::new_inner(path_to_fragment_shader, entry_point).block_on()
    }

    async fn get_device_and_queue() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("unable to create adapter from wgpu instance");

        let device_and_queue = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("unable to create device and queue from wgpu adapter");
        device_and_queue
    }

    fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            label: None,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        };
        device.create_texture(&texture_desc)
    }

    fn create_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
        let row_size = width * 4;
        let bytes_per_row = (row_size + 255) & !255;

        device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (bytes_per_row * height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }

    async fn new_inner(path_to_fragment_shader: &str, entry_point: &str) -> Self {
        let (device, queue) = Self::get_device_and_queue().await;

        let vertex_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/fullscreen_vertex.wgsl"));

        let fragment_shader_source =
            fs::read_to_string(path_to_fragment_shader).expect("Unable to read fragment shader");

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(fragment_shader_source.into()),
        });

        let creation_time = Instant::now();
        let width = 64u16;
        let height = 64u16;

        let texture = Self::create_texture(&device, width.into(), height.into());
        let output_buffer = Self::create_buffer(&device, width.into(), height.into());

        let shader_input = ShaderInput {
            time: creation_time.elapsed().as_secs_f32(),
            resolution: [width.into(), height.into()],
            padding: 0f32,
        };

        let shader_input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[shader_input]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let user_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[T::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &shader_input_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &user_data_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: Some("main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        WgpuBackend {
            device,
            queue,
            pipeline,
            creation_time,
            texture,
            output_buffer,
            shader_input_buffer,
            bind_group,
            width,
            height,
            _user_data: PhantomData,
        }
    }

    async fn execute_inner(&mut self, width: u16, height: u16, _user_data: &T) -> Vec<[u8; 4]> {
        if bytes_per_row(width) != bytes_per_row(self.width) || height != self.height {
            self.texture = Self::create_texture(&self.device, width.into(), height.into());
            self.output_buffer = Self::create_buffer(&self.device, width.into(), height.into());
        }
        let bytes_per_row = bytes_per_row(width);

        let texture_view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let render_target = wgpu::RenderPassColorAttachment {
            view: &texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        };

        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(render_target)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        command_encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row.into()),
                    rows_per_image: Some(height.into()),
                },
            },
            wgpu::Extent3d {
                width: width.into(),
                height: height.into(),
                depth_or_array_layers: 1,
            },
        );

        let elapsed = self.creation_time.elapsed().as_secs_f32();
        self.queue.write_buffer(
            &self.shader_input_buffer,
            0,
            bytemuck::cast_slice(&[ShaderInput {
                time: elapsed,
                resolution: [self.width.into(), self.height.into()],
                padding: 0f32,
            }]),
        );
        self.queue.submit(Some(command_encoder.finish()));

        let buffer_slice = self.output_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
            sender
                .send(r)
                .expect("unable to send buffer slice data to receiver");
        });
        self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        receiver
            .recv_async()
            .await
            .expect("unable to receive message all senders have been dropped")
            .expect("on unexpected error occured");
        let padded_buffer: Vec<[u8; 4]>;
        {
            let view = buffer_slice.get_mapped_range();
            padded_buffer = bytemuck::cast_slice(&view).to_vec();
        }
        self.output_buffer.unmap();
        let mut buffer: Vec<[u8; 4]> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let index = (y * (width + row_padding(width)) + x) as usize;
                let pixel = padded_buffer[index];
                buffer.push(pixel);
            }
        }
        buffer
    }
}

impl<T> TuiShaderBackend<T> for WgpuBackend<T>
where
    T: Copy + Clone + Default + bytemuck::Pod + bytemuck::Zeroable,
{
    fn execute(&mut self, width: u16, height: u16, user_data: &T) -> Vec<[u8; 4]> {
        self.execute_inner(width, height, user_data).block_on()
    }
}

impl Default for WgpuBackend<NoUserData> {
    fn default() -> Self {
        Self::new("src/shaders/default_fragment.wgsl", "magenta")
    }
}

fn bytes_per_row(width: u16) -> u16 {
    let row_size = width * 4;
    (row_size + 255) & !255
}

fn row_padding(width: u16) -> u16 {
    let row_size = width * 4;
    let bytes_per_row = bytes_per_row(width);
    (bytes_per_row - row_size) / 4
}
