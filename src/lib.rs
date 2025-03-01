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

use std::marker::PhantomData;
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
pub struct ShaderCanvas<T> {
    pub character_rule: CharacterRule,
    pub style_rule: StyleRule,
    pub entry_point: String,
    _user_data: PhantomData<T>,
}

impl<T> ShaderCanvas<T> {
    /// Creates a new instance of [`ShaderCanvas`].
    pub fn new() -> Self {
        Self {
            character_rule: CharacterRule::default(),
            style_rule: StyleRule::default(),
            entry_point: String::from("main"),
            _user_data: PhantomData,
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

    /// Sets the entry point of the fragment shader in the [`ShaderCanvas`].
    /// The default value is "main".
    #[must_use]
    pub fn entry_point(mut self, entry_point: &str) -> Self {
        self.entry_point = String::from(entry_point);
        self
    }
}

impl<T> Default for ShaderCanvas<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StatefulWidget for ShaderCanvas<T> {
    type State = ShaderCanvasState<T>;
    fn render(
        self,
        area: Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut ShaderCanvasState<T>,
    ) {
        let width = area.width;
        let height = area.height;
        let shader_input = ShaderInput::new(state.instant.elapsed().as_secs_f32(), width, height);
        let samples = state.backend.execute(&shader_input, &state.user_data);

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
    user_data: T,
    instant: Instant,
}

impl<T> ShaderCanvasState<T> {
    pub fn new<B: TuiShaderBackend<T> + 'static>(backend: B, user_data: T) -> Self {
        let backend = Box::new(backend);
        let creation_time = Instant::now();
        ShaderCanvasState {
            backend,
            user_data,
            instant: creation_time,
        }
    }
}

impl ShaderCanvasState {
    /// Creates a new [`ShaderCanvasState`] using [`WgpuBackend`] as it's
    /// [`TuiShaderBackend`].
    pub fn wgpu(path_to_fragment_shader: &str, entry_point: &str) -> Self {
        let backend = WgpuBackend::new(path_to_fragment_shader, entry_point);
        let user_data = NoUserData::default();
        Self::new(backend, user_data)
    }

    pub fn cpu<F>(callback: F) -> Self
    where
        F: Fn(u32, u32) -> Pixel + 'static,
    {
        let backend = CpuBackend::new(callback);
        let user_data = NoUserData::default();
        Self::new(backend, user_data)
    }

    pub fn builder() -> ShaderCanvasStateBuilder {
        ShaderCanvasStateBuilder::default()
    }
}

impl<T> ShaderCanvasState<T>
where
    T: Copy + Default + bytemuck::Pod + bytemuck::Zeroable,
{
    pub fn wgpu_with_user_data(
        path_to_fragment_shader: &str,
        entry_point: &str,
        user_data: T,
    ) -> Self {
        let backend = WgpuBackend::new(path_to_fragment_shader, entry_point);
        Self::new(backend, user_data)
    }
}

impl<T> ShaderCanvasState<T>
where
    T: 'static,
{
    pub fn cpu_with_user_data<F>(callback: F, user_data: T) -> Self
    where
        F: Fn(u32, u32, &T) -> Pixel + 'static,
    {
        let backend = CpuBackend::new_with_user_data(callback);
        Self::new(backend, user_data)
    }
}

impl Default for ShaderCanvasState {
    /// Creates a new [`ShaderCanvasState`] instance with a [`WgpuBackend`].
    fn default() -> Self {
        let backend = WgpuBackend::default();
        let user_data = NoUserData::default();
        Self::new(backend, user_data)
    }
}

#[derive(Default)]
pub struct ShaderCanvasStateBuilder<T = NoUserData> {
    backend: Option<Box<dyn TuiShaderBackend<T>>>,
    user_data: Option<T>,
    instant: Option<Instant>,
}

impl<T> ShaderCanvasStateBuilder<T> {
    pub fn with_backend<B: TuiShaderBackend<T> + 'static>(mut self, backend: B) -> Self {
        self.backend = Some(Box::new(backend));
        self
    }

    pub fn with_user_data(mut self, user_data: T) -> Self {
        self.user_data = Some(user_data);
        self
    }

    pub fn with_instant(mut self, instant: Instant) -> Self {
        self.instant = Some(instant);
        self
    }
}

impl ShaderCanvasStateBuilder {
    pub fn build(self) -> ShaderCanvasState {
        let backend = self
            .backend
            .unwrap_or_else(|| Box::new(WgpuBackend::default()));
        let user_data = NoUserData::default();
        let instant = self.instant.unwrap_or_else(|| Instant::now());
        ShaderCanvasState {
            backend,
            user_data,
            instant,
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderInput {
    // struct field order matters
    time: f32,
    padding: f32,
    resolution: [u32; 2],
}

impl ShaderInput {
    pub fn new(time: f32, width: u16, height: u16) -> Self {
        Self {
            time,
            padding: f32::default(),
            resolution: [width.into(), height.into()],
        }
    }
}

impl Default for ShaderInput {
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
        let raw_buffer = context.execute(&ShaderInput::default(), &NoUserData::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 255, 255]));
    }

    #[test]
    fn different_entry_points() {
        let mut context = WgpuBackend::new("src/shaders/default_fragment.wgsl", "green");
        let raw_buffer = context.execute(&ShaderInput::default(), &NoUserData::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[0, 255, 0, 255]));
    }

    #[test]
    fn cpu_backend() {
        fn cb(_x: u32, _y: u32) -> Pixel {
            [255, 0, 0, 255]
        }
        let mut context = CpuBackend::new(cb);
        let raw_buffer = context.execute(&ShaderInput::default(), &NoUserData::default());
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
                        if sample.x() == 0 {
                            ' '
                        } else {
                            '.'
                        }
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
