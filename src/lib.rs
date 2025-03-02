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
//! let mut terminal = ratatui::init();
//! let mut state = tui_shader::ShaderCanvasState::default();
//! let start_time = std::time::Instant::now();
//! while start_time.elapsed().as_secs() < 5 {
//!     terminal.draw(|frame| {
//!         frame.render_stateful_widget(tui_shader::ShaderCanvas::new(), frame.area(), &mut state);
//!     }).unwrap();
//! }
//! ratatui::restore();
//! ```

mod backend;

use std::time::Instant;

use backend::cpu::CpuBackend;
use backend::{NoUserData, TuiShaderBackend};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::StatefulWidget;

use crate::backend::wgpu::WgpuBackend;

/// `ShaderCanvas` is a unit struct which implements the `StatefulWidget` trait from Ratatui.
/// It holds the logic for applying the result of GPU computation to the `Buffer` struct which
/// Ratatui uses to display to the terminal.
///
/// ```rust,no_run
/// let mut terminal = ratatui::init();
/// let mut state = tui_shader::ShaderCanvasState::default();
/// terminal.draw(|frame| {
///     frame.render_stateful_widget(tui_shader::ShaderCanvas::new()
///         .style_rule(tui_shader::StyleRule::ColorFg),
///         frame.area(),
///         &mut state);
/// }).unwrap();
/// ratatui::restore();
/// ```

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShaderCanvas {
    pub character_rule: CharacterRule,
    pub style_rule: StyleRule,
    pub instant: Instant,
}

impl ShaderCanvas {
    /// Creates a new instance of [`ShaderCanvas`].
    pub fn new() -> Self {
        Self {
            character_rule: CharacterRule::default(),
            style_rule: StyleRule::default(),
            instant: Instant::now(),
        }
    }

    /// Applies a [`CharacterRule`] to a [`ShaderCanvas`].
    #[must_use]
    pub fn character_rule(mut self, character_rule: CharacterRule) -> Self {
        self.character_rule = character_rule;
        self
    }

    /// Applies a [`StyleRule`] to a [`ShaderCanvas`].
    #[must_use]
    pub fn style_rule(mut self, style_rule: StyleRule) -> Self {
        self.style_rule = style_rule;
        self
    }

    #[must_use]
    pub fn instant(mut self, instant: Instant) -> Self {
        self.instant = instant;
        self
    }
}

impl Default for ShaderCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for ShaderCanvas {
    type State = ShaderCanvasState;
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer, state: &mut ShaderCanvasState) {
        let width = area.width;
        let height = area.height;
        let ctx = ShaderContext::new(self.instant.elapsed().as_secs_f32(), width, height);
        let samples = state.backend.execute(ctx);

        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
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

/// State struct for [`ShaderCanvas`], it holds the [`TuiShaderBackend`].
pub struct ShaderCanvasState<T = NoUserData> {
    backend: Box<dyn TuiShaderBackend<T>>,
}

impl<T> ShaderCanvasState<T> {
    pub fn new<B: TuiShaderBackend<T> + 'static>(backend: B) -> Self {
        let backend = Box::new(backend);
        ShaderCanvasState { backend }
    }
}

impl ShaderCanvasState {
    /// Creates a new [`ShaderCanvasState`] using [`WgpuBackend`] as it's
    /// [`TuiShaderBackend`].
    pub fn wgpu<'a, T: Into<wgpu::ShaderModuleDescriptor<'a>>>(shader_desc: T) -> Self {
        let backend = WgpuBackend::new(shader_desc, None);
        Self::new(backend)
    }

    pub fn cpu<F>(callback: F) -> Self
    where
        F: Fn(u32, u32, ShaderContext) -> Pixel + 'static,
    {
        let backend = CpuBackend::new(callback);
        Self::new(backend)
    }
}

impl Default for ShaderCanvasState {
    /// Creates a new [`ShaderCanvasState`] instance with a [`WgpuBackend`].
    fn default() -> Self {
        let backend = WgpuBackend::default();
        Self::new(backend)
    }
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
/// [`CharacterRule::Always`] takes a single char and applies it to all Cells in the [`ShaderCanvas`].
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
    fn default_wgsl_context() {
        let mut context = WgpuBackend::default();
        let raw_buffer = context.execute(ShaderContext::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 255, 255]));
    }

    #[test]
    fn different_entry_points() {
        let mut context = WgpuBackend::new(
            wgpu::include_wgsl!("../src/shaders/default_fragment.wgsl"),
            Some("green"),
        );
        let raw_buffer = context.execute(ShaderContext::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[0, 255, 0, 255]));
    }

    #[test]
    fn cpu_backend() {
        fn red(_: u32, _: u32, _: ShaderContext) -> Pixel {
            [255, 0, 0, 255]
        }
        let mut context = CpuBackend::new(red);
        let raw_buffer = context.execute(ShaderContext::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 0, 255]));
    }

    #[test]
    fn character_rule_map() {
        let mut terminal = ratatui::Terminal::new(TestBackend::new(64, 64)).unwrap();
        let mut state = ShaderCanvasState::default();
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(
                    ShaderCanvas::new().character_rule(CharacterRule::Map(|sample| {
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
