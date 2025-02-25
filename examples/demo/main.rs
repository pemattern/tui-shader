fn main() {
    let mut terminal = ratatui::init();
    let mut state = tui_shader::ShaderCanvasState::new_with_options(
        "shaders/dither.wgsl",
        tui_shader::ShaderCanvasOptions {
            ..Default::default()
        },
    );

    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 20 {
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut state);
            })
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    ratatui::restore();
}
