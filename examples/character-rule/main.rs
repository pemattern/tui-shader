use tui_shader::{CharacterRule, ShaderCanvas, ShaderCanvasState, StyleRule};

fn main() {
    let mut terminal = ratatui::init();

    let char_map = CharacterRule::Map(|sample| if sample.r() > 127 { '@' } else { ' ' });
    let canvas = ShaderCanvas::new()
        .character_rule(char_map)
        .style_rule(StyleRule::ColorFg);

    let mut state = ShaderCanvasState::new(wgpu::include_wgsl!("../../shaders/voronoi.wgsl"));

    while state.get_instant().elapsed().as_secs() < 5 {
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(&canvas, frame.area(), &mut state);
            })
            .unwrap();
    }
    ratatui::restore();
}
