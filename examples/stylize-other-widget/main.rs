pub fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut state = tui_shader::ShaderCanvasState::new_with_options(
        "shaders/gradient.wgsl",
        // This sets the character for our ShaderCanvas to use to a Space,
        // which never displays the foreground color. This way we can use
        // the foreground coloring for a character that we specify in a
        // different widget
        tui_shader::ShaderCanvasOptions {
            character_rule: tui_shader::CharacterRule::Always(' '),
            ..Default::default()
        },
    );

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
            frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut state);
            frame.render_widget(
                ratatui::widgets::Paragraph::new(lorem_ipsum)
                    .wrap(ratatui::widgets::Wrap { trim: true }),
                frame.area(),
            );
        })?;
        if start_time.elapsed().as_secs() > 5 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    ratatui::restore();
    Ok(())
}
