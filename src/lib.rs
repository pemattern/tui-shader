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
//!         frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut state);
//!     }).unwrap();
//! }
//! ratatui::restore();
//! ```

mod wgpu_context;

use pollster::FutureExt;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::StatefulWidget;

use crate::wgpu_context::WgpuContext;

/// `ShaderCanvas` is a unit struct which implements the `StatefulWidget` trait from Ratatui.
/// It holds the logic for applying the result of GPU computation to the `Buffer` struct which
/// Ratatui uses to display to the terminal.
///
/// ```rust,no_run
/// let mut terminal = ratatui::init();
/// let mut state = tui_shader::ShaderCanvasState:default();
/// terminal.draw(|frame| {
///     frame.render_stateful_widget(ShaderCanvas, frame.area(), &mut state);
/// }
/// ratatui::restore();
/// ```
pub struct ShaderCanvas;

impl StatefulWidget for ShaderCanvas {
    type State = ShaderCanvasState;
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
        let width = area.width;
        let height = area.height;

        let raw_buffer = state.wgpu_context.execute(width, height).block_on();

        for y in 0..height {
            for x in 0..width {
                let index = (y * (width + WgpuContext::row_padding(width)) + x) as usize;
                let pixel = raw_buffer[index];
                let character = match state.options.character_rule {
                    CharacterRule::Always(character) => character,
                    CharacterRule::Map(map) => map(pixel.into()),
                };
                let color = Color::Rgb(pixel[0], pixel[1], pixel[2]);
                let style = match state.options.style_rule {
                    StyleRule::ColorFg => Style::new().fg(color),
                    StyleRule::ColorBg => Style::new().bg(color),
                    StyleRule::ColorFgAndBg => Style::new().fg(color).bg(color),
                    StyleRule::Map(map) => map(pixel.into()),
                };
                let cell = buf.cell_mut(Position::new(x, y)).unwrap();
                cell.set_style(style);
                cell.set_char(character);
            }
        }
    }
}

/// State struct for [`ShaderCanvas`].
///
/// This struct holds values that may want to be altered at runtime.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct ShaderCanvasState {
    wgpu_context: WgpuContext,
    options: ShaderCanvasOptions,
}

impl ShaderCanvasState {
    /// Creates a new [`ShaderCanvasState`] by passing in the path to the desired fragment shader.
    /// The shader must be written in WGSL. The [`ShaderCanvasOptions`] will be set to
    /// [`Self::default()`].
    pub fn new(path_to_fragment_shader: &str) -> Self {
        Self::new_with_options(path_to_fragment_shader, ShaderCanvasOptions::default())
    }

    /// Creates a new [`ShaderCanvasState`] by passing in the path to the desired fragment shader.
    /// The shader must be written in WGSL. The [`ShaderCanvasOptions`] can be customized.
    ///
    /// ```rust,no_run
    /// let state = tui_shader::ShaderCanvasState::new_with_options("path/to/shader.wgsl",
    ///     tui_shader::ShaderCanvasOptions {
    ///         style_rule: StyleRule::Fg,
    ///         entry_point: "fragment",
    ///         ..Default::default()
    ///     }
    /// );
    /// ```
    pub fn new_with_options(path_to_fragment_shader: &str, options: ShaderCanvasOptions) -> Self {
        Self {
            wgpu_context: WgpuContext::new(path_to_fragment_shader, &options.entry_point)
                .block_on(),
            options,
        }
    }
}

/// Contains options to customize the behaviour of the ShaderCanvas.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShaderCanvasOptions {
    pub character_rule: CharacterRule,
    pub style_rule: StyleRule,
    pub entry_point: String,
}

impl Default for ShaderCanvasOptions {
    fn default() -> Self {
        Self {
            character_rule: CharacterRule::default(),
            style_rule: StyleRule::default(),
            entry_point: String::from("main"),
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
    fn default() -> Self {
        Self::Always(' ')
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum StyleRule {
    ColorFg,
    #[default]
    ColorBg,
    ColorFgAndBg,
    Map(fn(Sample) -> Style),
}

pub struct Sample {
    value: [u8; 4],
    position: (u16, u16),
}

impl Sample {
    pub fn r(&self) -> u8 {
        self.value[0]
    }

    pub fn g(&self) -> u8 {
        self.value[1]
    }

    pub fn b(&self) -> u8 {
        self.value[2]
    }

    pub fn a(&self) -> u8 {
        self.value[3]
    }

    pub fn x(&self) -> u16 {
        self.position.0
    }

    pub fn y(&self) -> u16 {
        self.position.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_wgsl_context() {
        let mut context = WgpuContext::default();
        let raw_buffer = context.execute(64, 64).block_on();
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 255, 255]));
    }
}
