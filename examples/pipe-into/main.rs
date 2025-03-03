pub fn main() -> std::io::Result<()> {
    let mut s = String::new();
    {
        let mut stdin_lock = std::io::stdin().lock();
        std::io::Read::read_to_string(&mut stdin_lock, &mut s).unwrap();
    }
    let mut terminal = ratatui::init();
    let mut state = tui_shader::ShaderCanvasState::new(
        wgpu::include_wgsl!("../../shaders/gradient.wgsl"),
        None,
    );
    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 7 {
        terminal.draw(|frame| {
            frame.render_stateful_widget(
                tui_shader::ShaderCanvas::new().style_rule(tui_shader::StyleRule::ColorFg),
                frame.area(),
                &mut state,
            );
            frame.render_widget(ratatui::widgets::Paragraph::new(s.as_str()), frame.area());
        })?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    ratatui::restore();
    Ok(())
}
