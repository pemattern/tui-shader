use std::time::Instant;

use pollster::FutureExt;
use wgpu::util::DeviceExt;

use crate::{Pixel, bytes_per_row, context::ShaderContext};

const DEFAULT_SIZE: u32 = 64;

/// [`ShaderCanvasState`] holds the state to execute a render pass. It handles window/widget resizing automatically
/// and creates new textures and buffers when necessary.
#[derive(Debug, Clone)]
pub struct ShaderCanvasState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    output_buffer: wgpu::Buffer,
    ctx_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instant: Instant,
    width: u32,
    height: u32,
}

impl ShaderCanvasState {
    /// Creates a new [`ShaderCanvasState`] instance, without specifying an entry point. This means that
    /// the wgsl shader must define exactly one `@fragment` function.
    pub fn new<'a, S: Into<wgpu::ShaderModuleDescriptor<'a>>>(shader: S) -> Self {
        Self::new_inner(shader.into(), None).block_on()
    }

    /// Creates a new [`ShaderCanvasState`] instance with an entry point. This is necessary when your wgsl
    /// shader defines more than one `@fragment` function. In this case, the name of the function must be passed
    /// in.
    pub fn new_with_entry_point<'a, S: Into<wgpu::ShaderModuleDescriptor<'a>>>(
        shader: S,
        entry_point: &'a str,
    ) -> Self {
        Self::new_inner(shader.into(), Some(entry_point)).block_on()
    }

    #[allow(clippy::needless_lifetimes)]
    async fn new_inner<'a>(
        desc: wgpu::ShaderModuleDescriptor<'a>,
        entry_point: Option<&str>,
    ) -> Self {
        let (device, queue) = get_device_and_queue().await;

        let vertex_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/fullscreen_vertex.wgsl"));

        let fragment_shader = device.create_shader_module(desc);

        let texture = create_texture(&device, DEFAULT_SIZE, DEFAULT_SIZE);
        let output_buffer = create_buffer(&device, DEFAULT_SIZE, DEFAULT_SIZE);

        let ctx_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[ShaderContext::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &ctx_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
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
                entry_point,
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

        ShaderCanvasState {
            device,
            queue,
            pipeline,
            texture,
            output_buffer,
            ctx_buffer,
            bind_group,
            instant: Instant::now(),
            width: DEFAULT_SIZE,
            height: DEFAULT_SIZE,
        }
    }

    pub(crate) fn execute(&mut self, ctx: ShaderContext) -> Vec<Pixel> {
        self.execute_inner(ctx).block_on()
    }

    async fn execute_inner(&mut self, ctx: ShaderContext) -> Vec<Pixel> {
        let width = ctx.width();
        let height = ctx.height();
        if bytes_per_row(width) != bytes_per_row(self.width) || height != self.height {
            self.texture = create_texture(&self.device, width, height);
            self.output_buffer = create_buffer(&self.device, width, height);
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
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue
            .write_buffer(&self.ctx_buffer, 0, bytemuck::cast_slice(&[ctx]));
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
        let padded_buffer: Vec<Pixel>;
        {
            let view = buffer_slice.get_mapped_range();
            padded_buffer = bytemuck::cast_slice(&view).to_vec();
        }
        padded_buffer
    }

    /// Sets the [`ShaderCanvasState`]'s [`Instant`]. This can be useful if you want to sync the time input variable
    /// across multiple fragment shaders, or a specific [`Instant`] is required.
    pub fn set_instant(mut self, instant: Instant) {
        self.instant = instant;
    }

    /// Gets the [`ShaderCanvasState`]'s [`Instant`].
    pub fn get_instant(&self) -> Instant {
        self.instant
    }
}

impl Default for ShaderCanvasState {
    fn default() -> Self {
        Self::new(wgpu::include_wgsl!("shaders/default_fragment.wgsl"))
    }
}

async fn get_device_and_queue() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("unable to create adapter from wgpu instance");

    adapter
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
        .expect("unable to create device and queue from wgpu adapter")
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
