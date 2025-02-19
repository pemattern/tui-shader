pub mod sample;
mod wgpu_context;

use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::widgets::StatefulWidget;
use sample::Sample;

use crate::wgpu_context::WgpuContext;

pub struct ShaderCanvas;

impl StatefulWidget for ShaderCanvas {
    type State = ShaderCanvasState;
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
        let width = area.width;
        let height = area.height;

        let raw_buffer = state.wgpu_context.execute(width, height);

        for y in 0..height {
            for x in 0..width {
                let index = (y * (width + WgpuContext::row_padding(width)) + x) as usize;
                let pixel = raw_buffer[index];
                let character = match state.options.character_rule {
                    CharacterRule::Always(character) => character,
                    CharacterRule::Map(map) => map(pixel.into()),
                };
                let color = Color::Rgb(pixel[0], pixel[1], pixel[2]);
                let (fg_color, bg_color) = match state.options.color_rule {
                    ColorRule::Fg => (Some(color), None),
                    ColorRule::Bg => (None, Some(color)),
                    ColorRule::FgAndBg => (Some(color), Some(color)),
                    ColorRule::Map(map) => map(pixel.into()),
                };
                let cell = buf.cell_mut(Position::new(x, y)).unwrap();
                if let Some(color) = fg_color {
                    cell.set_fg(color);
                }
                if let Some(color) = bg_color {
                    cell.set_bg(color);
                }
                cell.set_char(character);
            }
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct ShaderCanvasState {
    wgpu_context: WgpuContext,
    options: ShaderCanvasOptions,
}

impl ShaderCanvasState {
    pub fn new(path_to_fragment_shader: &str) -> Self {
        Self::new_with_options(path_to_fragment_shader, ShaderCanvasOptions::default())
    }

    pub fn new_with_options(path_to_fragment_shader: &str, options: ShaderCanvasOptions) -> Self {
        Self {
            wgpu_context: WgpuContext::new(path_to_fragment_shader, &options.entry_point),
            options,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShaderCanvasOptions {
    pub character_rule: CharacterRule,
    pub color_rule: ColorRule,
    pub entry_point: String,
}

impl Default for ShaderCanvasOptions {
    fn default() -> Self {
        Self {
            character_rule: CharacterRule::default(),
            color_rule: ColorRule::default(),
            entry_point: String::from("main"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CharacterRule {
    Always(char),
    Map(fn(Sample) -> char),
}

impl Default for CharacterRule {
    fn default() -> Self {
        Self::Always('█')
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ColorRule {
    Fg,
    Bg,
    FgAndBg,
    Map(fn(Sample) -> (Option<Color>, Option<Color>)),
}

impl Default for ColorRule {
    fn default() -> Self {
        ColorRule::Fg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_wgsl_context() {
        let mut context = WgpuContext::default();
        let raw_buffer = context.execute(64, 64);
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 255, 255]));
    }
}
