pub fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut state = tui_shader::ShaderCanvasState::new("shaders/voronoi.wgsl");

    let start_time = std::time::Instant::now();
    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut state);
        })?;
        if start_time.elapsed().as_secs() > 5 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    ratatui::restore();
    Ok(())
}
