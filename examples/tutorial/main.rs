use tui_shader::{ShaderCanvas, ShaderCanvasState, WgslShader};

fn main() {
    let mut terminal = ratatui::init();
    let shader = WgslShader::Path("shaders/tutorial.wgsl");
    let mut state = ShaderCanvasState::new(shader);
    while state.get_instant().elapsed().as_secs() < 7 {
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(ShaderCanvas::new(), frame.area(), &mut state);
            })
            .unwrap();
    }
    ratatui::restore();
}
