use ratatui::{
    crossterm,
    layout::{Constraint, Layout, Margin},
    widgets::{Block, Paragraph, Wrap},
};
use tui_shader::{ShaderCanvas, ShaderCanvasState, WgslShader};
use tui_textarea::{Input, Key, TextArea};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();

    let source = std::fs::read_to_string("shaders/gradient.wgsl").unwrap();
    let mut state = ShaderCanvasState::new(WgslShader::Source(source.as_str())).unwrap();
    let canvas = ShaderCanvas::default();

    let mut textarea = TextArea::new(source.lines().map(|s| s.to_string()).collect());

    let mut error_message: Option<String> = None;

    loop {
        terminal.draw(|frame| {
            let [editor_area, shader_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Min(1)])
                    .areas(frame.area());

            frame.render_widget(
                Block::bordered().title(" Editor | <F5> Reload | <Esc> Quit "),
                editor_area,
            );
            frame.render_widget(Block::bordered().title(" Preview "), shader_area);
            frame.render_widget(&textarea, editor_area.inner(Margin::new(1, 1)));
            if error_message.is_none() {
                frame.render_stateful_widget(
                    &canvas,
                    shader_area.inner(Margin::new(1, 1)),
                    &mut state,
                );
            } else {
                frame.render_widget(
                    Paragraph::new(error_message.clone().unwrap().as_str())
                        .wrap(Wrap { trim: false }),
                    shader_area.inner(Margin::new(1, 1)),
                );
            }
        })?;

        if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(20)) {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                Input { key: Key::F(5), .. } => {
                    error_message = None;
                    state = ShaderCanvasState::new(WgslShader::Source(
                        textarea.lines().join("\n").as_str(),
                    ))
                    .unwrap_or_else(|e| {
                        error_message = Some(e.to_string());
                        state
                    });
                }
                input => {
                    textarea.input(input);
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}
