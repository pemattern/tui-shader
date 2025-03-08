pub fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut state =
        tui_shader::ShaderCanvasState::new(wgpu::include_wgsl!("../../shaders/voronoi.wgsl"))
            .unwrap();

    let start_time = std::time::Instant::now();
    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(tui_shader::ShaderCanvas::new(), frame.area(), &mut state);
        })?;
        if start_time.elapsed().as_secs() > 5 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    ratatui::restore();
    Ok(())
}
