pub fn main() -> std::io::Result<()> {
    let mut s = String::new();
    {
        let mut stdin_lock = std::io::stdin().lock();
        std::io::Read::read_to_string(&mut stdin_lock, &mut s).unwrap();
    }

    let mut terminal = ratatui::init();
    let mut state = tui_shader::ShaderCanvasState::new("shaders/gradient.wgsl");

    let start_time = std::time::Instant::now();
    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut state);
            frame.render_widget(ratatui::widgets::Paragraph::new(s.as_str()), frame.area());
        })?;
        if start_time.elapsed().as_secs() > 5 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    ratatui::restore();
    Ok(())
}
