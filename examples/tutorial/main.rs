use ratatui::style::Style;
use tui_shader::{ShaderCanvas, ShaderCanvasState, WgslShader};

fn main() {
    let mut terminal = ratatui::init();

    let balloon_shader = WgslShader::Path("shaders/tutorial.wgsl");
    let mut balloon_state = ShaderCanvasState::new(balloon_shader);
    let balloon_canvas = ShaderCanvas::new().style_rule(tui_shader::StyleRule::Map(|sample| {
        let brightness = (sample.r() as f32 + sample.g() as f32 + sample.b() as f32) / 3.0;
        if brightness > 127.0 {
            Style::new().bg(sample.color())
        } else {
            Style::default()
        }
    }));

    let bg_shader = WgslShader::Path("shaders/gradient.wgsl");
    let mut bg_state = ShaderCanvasState::new(bg_shader);
    let bg_canvas = ShaderCanvas::new();

    while balloon_state.get_instant().elapsed().as_secs() < 7 {
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(&bg_canvas, frame.area(), &mut bg_state);
                frame.render_stateful_widget(&balloon_canvas, frame.area(), &mut balloon_state);
            })
            .unwrap();
    }
    ratatui::restore();
}
