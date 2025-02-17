use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::widgets::StatefulWidget;

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
                let character = match state.character_rule {
                    CharacterRule::Always(character) => character,
                    CharacterRule::Map(map) => map(pixel),
                };
                buf.cell_mut(Position::new(x, y))
                    .unwrap()
                    .set_fg(Color::Rgb(pixel[0], pixel[1], pixel[2]))
                    .set_char(character);
            }
        }
    }
}

pub struct ShaderCanvasState {
    wgpu_context: WgpuContext,
    character_rule: CharacterRule,
}

impl ShaderCanvasState {
    pub fn new(path_to_fragment_shader: &str) -> Self {
        Self {
            wgpu_context: WgpuContext::new(path_to_fragment_shader),
            character_rule: CharacterRule::default(),
        }
    }
}

pub enum CharacterRule {
    Always(char),
    Map(fn([u8; 4]) -> char),
}

impl Default for CharacterRule {
    fn default() -> Self {
        Self::Always('█')
    }
}
