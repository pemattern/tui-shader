use std::{io::Result, thread::sleep, time::Duration};

use ratatui::{
    layout::Margin,
    style::{Color, Style},
};
use tui_big_text::{BigText, PixelSize};
use tui_shader::{ShaderCanvas, ShaderCanvasState, StyleRule};
use wgpu::include_wgsl;

const DARK_COLOR: Color = Color::Rgb(75, 71, 92);
const LIGHT_COLOR: Color = Color::Rgb(215, 222, 220);

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut state = ShaderCanvasState::new(include_wgsl!("../../shaders/dither.wgsl"));
    let style_rule: StyleRule = StyleRule::Map(|sample| {
        if sample.r() > 127 {
            Style::new().fg(DARK_COLOR).bg(LIGHT_COLOR)
        } else {
            Style::new().fg(LIGHT_COLOR).bg(DARK_COLOR)
        }
    });
    let canvas = ShaderCanvas::new().style_rule(style_rule);
    while state.get_instant().elapsed().as_secs() < 20 {
        terminal.draw(|frame| {
            frame.render_stateful_widget(&canvas, frame.area(), &mut state);
            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::Full)
                    .lines(vec!["tui-shader".into()])
                    .centered()
                    .build(),
                frame.area().inner(Margin::new(4, 12)),
            );
        })?;
        sleep(Duration::from_millis(20));
    }

    ratatui::restore();
    Ok(())
}
