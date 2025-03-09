pub fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut state =
        tui_shader::ShaderCanvasState::new(wgpu::include_wgsl!("../../shaders/gradient.wgsl"))
            .unwrap();
    const STYLE_RULE: tui_shader::StyleRule = tui_shader::StyleRule::Map(|sample| {
        let color = sample.color();
        let sum = sample.r() as u16 + sample.g() as u16 + sample.b() as u16;
        if sum > 300 {
            ratatui::style::Style::new()
                .bg(color)
                .fg(ratatui::style::Color::Black)
        } else {
            ratatui::style::Style::new().fg(color)
        }
    });

    let lorem_ipsum = r#"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor
        invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam
        et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est
        Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed
        diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.
        At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea
        takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscingi
        elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam
        voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no
        sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur
        sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat,
        sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd
        gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet
        consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna
        aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet
        clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."#;

    let start_time = std::time::Instant::now();
    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(
                tui_shader::ShaderCanvas::new().style_rule(STYLE_RULE),
                frame.area(),
                &mut state,
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(lorem_ipsum)
                    .wrap(ratatui::widgets::Wrap { trim: true }),
                frame.area(),
            );
        })?;
        if start_time.elapsed().as_secs() > 7 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    ratatui::restore();
    Ok(())
}
