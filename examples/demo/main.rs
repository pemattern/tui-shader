const DARK_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(75, 71, 92);
const LIGHT_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(215, 222, 220);

fn main() {
    let mut terminal = ratatui::init();
    let mut state = tui_shader::ShaderCanvasState::wgpu("shaders/dither.wgsl", "main");
    const STYLE_RULE: tui_shader::StyleRule = tui_shader::StyleRule::Map(|sample| {
        if sample.r() > 127 {
            ratatui::style::Style::new().fg(DARK_COLOR).bg(LIGHT_COLOR)
        } else {
            ratatui::style::Style::new().fg(LIGHT_COLOR).bg(DARK_COLOR)
        }
    });

    let start_time = std::time::Instant::now();
    // run at 128x32 cells
    while start_time.elapsed().as_secs() < 20 {
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(
                    tui_shader::ShaderCanvas::new().style_rule(STYLE_RULE),
                    frame.area(),
                    &mut state,
                );
                frame.render_widget(
                    tui_big_text::BigText::builder()
                        .pixel_size(tui_big_text::PixelSize::Full)
                        .lines(vec!["tui-shader".into()])
                        .centered()
                        .build(),
                    frame.area().inner(ratatui::layout::Margin::new(4, 12)),
                );
            })
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    ratatui::restore();
}
