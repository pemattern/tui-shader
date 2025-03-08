use ratatui::{
    crossterm,
    layout::{Constraint, Layout, Margin},
    widgets::Block,
};
use tui_shader::{ShaderCanvas, ShaderCanvasState, WgslShader};
use tui_textarea::{Input, Key, TextArea};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();

    let source = std::fs::read_to_string("shaders/gradient.wgsl").unwrap();
    let mut state = ShaderCanvasState::new(WgslShader::Source(source.as_str()));
    let canvas = ShaderCanvas::default();

    let mut textarea = TextArea::new(source.lines().map(|s| s.to_string()).collect());

    loop {
        terminal.draw(|frame| {
            let [editor_area, shader_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Min(1)])
                    .areas(frame.area());

            frame.render_widget(Block::bordered().title("Editor"), editor_area);
            frame.render_widget(Block::bordered().title("Preview (F5 to reload)"), shader_area);
            frame.render_widget(&textarea, editor_area.inner(Margin::new(1, 1)));
            frame.render_stateful_widget(&canvas, shader_area.inner(Margin::new(1, 1)), &mut state);
        })?;

        if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(20)) {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                Input { key: Key::F(5),.. }  => state = reload_state(textarea.lines()),
                input => {
                    textarea.input(input);
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn reload_state(lines: &[String]) -> ShaderCanvasState {
    let source = lines.join("\n");
    return ShaderCanvasState::new(WgslShader::Source(source.as_str()));    
}
