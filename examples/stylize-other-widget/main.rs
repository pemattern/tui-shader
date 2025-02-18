use ratatui::{
    crossterm::event::Event,
    widgets::{Paragraph, Wrap},
};
use tui_shader::shader_canvas::{ShaderCanvas, ShaderCanvasOptions, ShaderCanvasState};

enum Message {
    Redraw,
    Exit,
}

pub fn main() -> std::io::Result<()> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let input_sender = sender.clone();
    std::thread::spawn(move || loop {
        match ratatui::crossterm::event::read().unwrap() {
            Event::Key(_) => input_sender.send(Message::Exit).unwrap(),
            Event::Resize(_, _) => input_sender.send(Message::Redraw).unwrap(),
            _ => {}
        }
    });

    let redraw_sender = sender.clone();
    std::thread::spawn(move || {
        let mut last_tick = std::time::Instant::now();
        loop {
            if last_tick.elapsed().as_millis() > 16 {
                redraw_sender.send(Message::Redraw).unwrap();
                last_tick = std::time::Instant::now();
            }
        }
    });

    let mut terminal = ratatui::init();
    let mut state = ShaderCanvasState::new_with_options(
        "shaders/gradient.wgsl",
        // This sets the character for our ShaderCanvas to use to a Space,
        // which never displays the foreground color. This way we can use
        // the foreground coloring for a character that we specify in a
        // different widget
        ShaderCanvasOptions {
            character_rule: tui_shader::shader_canvas::CharacterRule::Always(' '),
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

    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(ShaderCanvas, frame.area(), &mut state);
            frame.render_widget(
                Paragraph::new(lorem_ipsum).wrap(Wrap { trim: true }),
                frame.area(),
            );
        })?;
        match receiver.recv().unwrap() {
            Message::Redraw => {}
            Message::Exit => break,
        }
    }
    ratatui::restore();
    Ok(())
}
