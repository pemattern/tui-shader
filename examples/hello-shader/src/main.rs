use ratatui::crossterm::event::Event;
use tui_shader::shader_canvas::{ShaderCanvas, ShaderCanvasState};

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
    let mut state = ShaderCanvasState::new("examples/hello-shader/src/shader.wgsl");
    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(ShaderCanvas, frame.area(), &mut state);
        })?;
        match receiver.recv().unwrap() {
            Message::Redraw => {}
            Message::Exit => break,
        }
    }
    ratatui::restore();
    Ok(())
}
