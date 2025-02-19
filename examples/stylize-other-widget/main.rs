use ratatui::style::Stylize;

pub fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut fg_shader_state = tui_shader::ShaderCanvasState::new_with_options(
        "shaders/voronoi.wgsl",
        tui_shader::ShaderCanvasOptions {
            character_rule: tui_shader::CharacterRule::Always(' '),
            ..Default::default()
        },
    );
    let mut bg_shader_state = tui_shader::ShaderCanvasState::new_with_options(
        "shaders/starlight.wgsl", 
        tui_shader::ShaderCanvasOptions {
            character_rule: tui_shader::CharacterRule::Always(' '),
            color_rule: tui_shader::ColorRule::Bg,
            ..Default::default()
        });
    let mut list_state = ratatui::widgets::ListState::default();
    *list_state.selected_mut() = Some(5);
    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 7 {
        terminal.draw(|frame| {
            frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut fg_shader_state);
            frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut bg_shader_state);
            frame.render_stateful_widget(
                ratatui::widgets::List::new([
                    "hella data",
                    "this is some important stuff...",
                    "very good entry",
                    "ok, now we're getting serious",
                    "butter",
                    "2shader4me",
                ])
                .highlight_style(ratatui::style::Style::new().reversed())
                .block(ratatui::widgets::Block::bordered()),
                frame.area(),
                &mut list_state,
            );
        })?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    ratatui::restore();
    Ok(())
}
