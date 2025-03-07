use std::{
    io::{Read, Result, stdin},
    thread::sleep,
    time::{Duration, Instant},
};

use ratatui::widgets::Paragraph;
use tui_shader::{ShaderCanvasState, StyleRule};
use wgpu::include_wgsl;

pub fn main() -> Result<()> {
    let mut s = String::new();
    {
        let mut stdin_lock = stdin().lock();
        Read::read_to_string(&mut stdin_lock, &mut s).unwrap();
    }
    let mut terminal = ratatui::init();
    let mut state = ShaderCanvasState::new(include_wgsl!("../../shaders/gradient.wgsl"));
    let start_time = Instant::now();
    while start_time.elapsed().as_secs() < 7 {
        terminal.draw(|frame| {
            frame.render_stateful_widget(
                tui_shader::ShaderCanvas::new().style_rule(StyleRule::ColorFg),
                frame.area(),
                &mut state,
            );
            frame.render_widget(Paragraph::new(s.as_str()), frame.area());
        })?;
        sleep(Duration::from_millis(10));
    }
    ratatui::restore();
    Ok(())
}
