use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::StatefulWidget;

use crate::context::ShaderContext;
use crate::state::ShaderCanvasState;
use crate::style::{CharacterRule, StyleRule};
use crate::{Sample, row_padding};

/// [`ShaderCanvas`] implements the [`StatefulWidget`] trait from Ratatui.
/// It holds the logic for applying the result of GPU computation to the [`Buffer`] struct which
/// Ratatui uses to display to the terminal.
///
/// ```rust,no_run
/// # use tui_shader::{ShaderCanvas, ShaderCanvasState, StyleRule};
/// let mut terminal = ratatui::init();
/// let mut state = ShaderCanvasState::default();
/// terminal.draw(|frame| {
///     frame.render_stateful_widget(ShaderCanvas::new(),
///         frame.area(),
///         &mut state);
/// }).unwrap();
/// ratatui::restore();
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShaderCanvas {
    pub character_rule: CharacterRule,
    pub style_rule: StyleRule,
}

impl ShaderCanvas {
    /// Creates a new instance of [`ShaderCanvas`]. Equivalent to [`ShaderCanvas::default()`]
    pub fn new() -> Self {
        Self {
            character_rule: CharacterRule::default(),
            style_rule: StyleRule::default(),
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
}

impl Default for ShaderCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for ShaderCanvas {
    type State = ShaderCanvasState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

impl StatefulWidget for &ShaderCanvas {
    type State = ShaderCanvasState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let width = area.width;
        let height = area.height;
        let time = state.get_instant().elapsed().as_secs_f32();
        let ctx = ShaderContext::new(time, area);
        let samples = state.execute(ctx);

        for y in 0..height {
            for x in 0..width {
                let index = (y * (width + row_padding(width.into()) as u16) + x) as usize;
                let value = samples[index];
                let position = (x, y);
                let uv = (x as f32 / width as f32, y as f32 / height as f32);
                let character = match self.character_rule {
                    CharacterRule::Always(character) => character,
                    CharacterRule::Map(map) => map(Sample::new(value, position, uv)),
                };
                let color = Color::Rgb(value[0], value[1], value[2]);
                let style = match self.style_rule {
                    StyleRule::ColorFg => Style::new().fg(color),
                    StyleRule::ColorBg => Style::new().bg(color),
                    StyleRule::Map(map) => map(Sample::new(value, position, uv)),
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
