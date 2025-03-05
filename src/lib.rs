//! The `tui-shader` crate enables GPU accelerated styling for [`Ratatui`](https://ratatui.rs)
//! based applications.
//!
//! Computing styles at runtime can be expensive when run on the CPU, despite the
//! small "resolution" of cells in a terminal window. Utilizing the power of the
//! GPU helps us update the styling in the terminal at considerably higher framerates.
//!
//! ## Quickstart
//!
//! Add `ratatui` and `tui-shader` as dependencies to your Corgo.toml:
//!
//! ```shell
//! cargo add ratatui tui-shader
//! ```
//!
//! Then create a new application:
//!
//! ```rust,no_run
//! # use tui_shader::{Shader, ShaderState};
//! let mut terminal = ratatui::init();
//! let mut state = ShaderState::default();
//! while state.get_instant().elapsed().as_secs() < 5 {
//!     terminal.draw(|frame| {
//!         frame.render_stateful_widget(tui_shader::Shader::new(), frame.area(), &mut state);
//!     }).unwrap();
//! }
//! ratatui::restore();
//! ```

use pollster::FutureExt as _;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::StatefulWidget;
use std::time::Instant;
use wgpu::util::DeviceExt;

const DEFAULT_SIZE: u32 = 64;

/// `Shader` is a struct which implements the `StatefulWidget` trait from Ratatui.
/// It holds the logic for applying the result of GPU computation to the `Buffer` struct which
/// Ratatui uses to display to the terminal.
///
/// ```rust,no_run
/// # use tui_shader::{Shader, ShaderState, StyleRule};
/// let mut terminal = ratatui::init();
/// let mut state = ShaderState::default();
/// terminal.draw(|frame| {
///     frame.render_stateful_widget(Shader::new()
///         .style_rule(StyleRule::ColorFg),
///         frame.area(),
///         &mut state);
/// }).unwrap();
/// ratatui::restore();
/// ```

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Shader {
    pub character_rule: CharacterRule,
    pub style_rule: StyleRule,
}

impl Shader {
    /// Creates a new instance of [`Shader`].
    pub fn new() -> Self {
        Self {
            character_rule: CharacterRule::default(),
            style_rule: StyleRule::default(),
        }
    }

    /// Applies a [`CharacterRule`] to a [`Shader`].
    #[must_use]
    pub fn character_rule(mut self, character_rule: CharacterRule) -> Self {
        self.character_rule = character_rule;
        self
    }

    /// Applies a [`StyleRule`] to a [`Shader`].
    #[must_use]
    pub fn style_rule(mut self, style_rule: StyleRule) -> Self {
        self.style_rule = style_rule;
        self
    }
}

impl Default for Shader {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for Shader {
    type State = ShaderState;
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

impl StatefulWidget for &Shader {
    type State = ShaderState;
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
        let width = area.width;
        let height = area.height;
        let time = state.instant.elapsed().as_secs_f32();
        let ctx = ShaderContext::new(time, width, height);
        let samples = state.execute(ctx);

        for y in 0..height {
            for x in 0..width {
                let index = (y * (width + row_padding(width.into()) as u16) + x) as usize;
                // let index = (y * width + x) as usize;
                let value = samples[index];
                let position = (x, y);
                let character = match self.character_rule {
                    CharacterRule::Always(character) => character,
                    CharacterRule::Map(map) => map(Sample::new(value, position)),
                };
                let color = Color::Rgb(value[0], value[1], value[2]);
                let style = match self.style_rule {
                    StyleRule::ColorFg => Style::new().fg(color),
                    StyleRule::ColorBg => Style::new().bg(color),
                    StyleRule::ColorFgAndBg => Style::new().fg(color).bg(color),
                    StyleRule::Map(map) => map(Sample::new(value, position)),
                };
                let cell = buf
                    .cell_mut(Position::new(x, y))
                    .expect("unable to get cell");
                cell.set_style(style);
                cell.set_char(character);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderState {
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

impl ShaderState {
    pub fn new<'a, S: Into<wgpu::ShaderModuleDescriptor<'a>>>(shader: S) -> Self {
        Self::new_inner(shader.into(), None).block_on()
    }

    pub fn new_with_entry_point<'a, S: Into<wgpu::ShaderModuleDescriptor<'a>>>(
        shader: S,
        entry_point: &'a str,
    ) -> Self {
        Self::new_inner(shader.into(), Some(entry_point)).block_on()
    }

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

        ShaderState {
            device,
            queue,
            pipeline,
            texture,
            output_buffer,
            ctx_buffer,
            bind_group,
            instant: Instant::now(),
            width: DEFAULT_SIZE.into(),
            height: DEFAULT_SIZE.into(),
        }
    }

    pub fn get_instant(&self) -> Instant {
        self.instant
    }

    fn execute(&mut self, ctx: ShaderContext) -> Vec<Pixel> {
        self.execute_inner(ctx).block_on()
    }

    async fn execute_inner(&mut self, ctx: ShaderContext) -> Vec<Pixel> {
        let width = ctx.resolution[0];
        let height = ctx.resolution[1];
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

    pub fn instant(mut self, instant: Instant) -> Self {
        self.instant = instant;
        self
    }
}

impl Default for ShaderState {
    fn default() -> Self {
        Self::new(wgpu::include_wgsl!("shaders/default_fragment.wgsl"))
    }
}

fn bytes_per_row(width: u32) -> u32 {
    let row_size = width * 4;
    (row_size + 255) & !255
}

fn row_padding(width: u32) -> u32 {
    let row_size = width * 4;
    let bytes_per_row = bytes_per_row(width);
    (bytes_per_row - row_size) / 4
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

#[repr(C)]
#[derive(Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderContext {
    // struct field order matters
    time: f32,
    padding: f32,
    resolution: [u32; 2],
}

impl ShaderContext {
    pub fn new(time: f32, width: u16, height: u16) -> Self {
        Self {
            time,
            padding: f32::default(),
            resolution: [width.into(), height.into()],
        }
    }
}

impl Default for ShaderContext {
    fn default() -> Self {
        Self {
            time: 0.0,
            padding: 0.0,
            resolution: [64, 64],
        }
    }
}

/// Determines which character to use for Cell.
/// [`CharacterRule::Always`] takes a single char and applies it to all Cells in the [`Shader`].
/// [`CharacterRule::Map`] takes a function as an argument and allows you to map the input [`Sample`] to
/// a character. For example, one might use the transparency value from the shader ([Sample::a]) and map
/// it to a different character depending on the value.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CharacterRule {
    Always(char),
    Map(fn(Sample) -> char),
}

impl Default for CharacterRule {
    /// Returns `Self::Always(' ')`
    fn default() -> Self {
        Self::Always(' ')
    }
}

/// Determines how to style a Cell.
/// [`StyleRule::ColorFg`] only applies the color from the shader to the foreground of the Cell.
/// [`StyleRule::ColorBg`] only applies the color from the shader to the background of the Cell. This
/// is the default value.
/// [`StyleRule::ColorFgAndBg`] applies the color from the shader to the foreground and background of the Cell.
/// [`StyleRule::Map`] takes a function as an argument and allows you to map the input [`Sample`] to
/// a Style. For example, one might use the transparency value from the shader ([Sample::a]) and set
/// a cutoff for switching between bold and non-bold text.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum StyleRule {
    ColorFg,
    #[default]
    ColorBg,
    ColorFgAndBg,
    Map(fn(Sample) -> Style),
}

/// Primarily used in [`CharacterRule::Map`] and [`StyleRule::Map`], it provides access to a Cells color and position
/// allowing to map the output of the shader to more complex behaviour.
pub struct Sample {
    pixel: Pixel,
    position: (u16, u16),
}

impl Sample {
    fn new(pixel: Pixel, position: (u16, u16)) -> Self {
        Self { pixel, position }
    }

    /// The red channel of the [`Sample`]
    pub fn r(&self) -> u8 {
        self.pixel[0]
    }

    /// The green channel of the [`Sample`]
    pub fn g(&self) -> u8 {
        self.pixel[1]
    }

    /// The blue channel of the [`Sample`]
    pub fn b(&self) -> u8 {
        self.pixel[2]
    }

    /// The alpha channel of the [`Sample`]
    pub fn a(&self) -> u8 {
        self.pixel[3]
    }

    /// The x coordinate of the [`Sample`]
    pub fn x(&self) -> u16 {
        self.position.0
    }

    /// The y coordinate of the [`Sample`]
    pub fn y(&self) -> u16 {
        self.position.1
    }
}

pub type Pixel = [u8; 4];

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn default_state() {
        let mut state = ShaderState::default();
        let raw_buffer = state.execute(ShaderContext::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 255, 255]));
    }

    #[test]
    fn different_entry_points() {
        let mut state = ShaderState::new_with_entry_point(
            wgpu::include_wgsl!("shaders/test_fragment.wgsl"),
            "green",
        );
        let raw_buffer = state.execute(ShaderContext::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[0, 255, 0, 255]));
    }

    #[test]
    fn character_rule_map() {
        let mut terminal = ratatui::Terminal::new(TestBackend::new(64, 64)).unwrap();
        let mut state = ShaderState::default();
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(
                    Shader::new().character_rule(CharacterRule::Map(|sample| {
                        if sample.x() == 0 { ' ' } else { '.' }
                    })),
                    frame.area(),
                    &mut state,
                );
                let buffer = frame.buffer_mut();
                for x in 0..buffer.area.width {
                    for y in 0..buffer.area.height {
                        if x == 0 {
                            assert_eq!(buffer.cell_mut(Position::new(x, y)).unwrap().symbol(), " ");
                        } else {
                            assert_eq!(buffer.cell_mut(Position::new(x, y)).unwrap().symbol(), ".");
                        }
                    }
                }
            })
            .unwrap();
        ratatui::restore();
    }
}
