use rand::seq::{IndexedMutRandom, IndexedRandom};
use ratatui::style::Stylize;

fn main() {
    let mut terminal = ratatui::init();
    let mut state = tui_shader::ShaderCanvasState::new_with_options(
        "shaders/dither.wgsl",
        tui_shader::ShaderCanvasOptions {
            ..Default::default()
        },
    );

    let text = "In a world where time is unstable, Kai, a young scavenger, stumbles upon an ancient time-worn gauntlet buried beneath the ruins of a forgotten city. The moment he touches it, the gauntlet fuses to his arm, and suddenly, fragments of the past and future begin flickering around him. He learns that the gauntlet was once wielded by the Chrono Guardians, protectors of the timeline, but was lost when a rogue Guardian, Veyra, shattered time itself. Now, past and future realities collide, creating monstrous anomalies and cities frozen in time loops. With each use of the gauntlet, Kai glimpses memories of the past—and the terrifying future Veyra seeks to create. As he navigates a collapsing world, he must master time-bending abilities, gather allies from different eras, and confront Veyra before time fractures beyond repair. But with each jump through time, one question lingers: Is he truly saving time, or becoming the very force that will destroy it?";

    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 20 {
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(tui_shader::ShaderCanvas, frame.area(), &mut state);
                let block_area = frame.area().inner(ratatui::layout::Margin::new(8, 4));
                frame.render_widget(
                    ratatui::widgets::Block::new()
                        .bg(ratatui::style::Color::Rgb(75, 71, 92))
                        .fg(ratatui::style::Color::Rgb(215, 222, 220)),
                    block_area,
                );
                frame.render_widget(
                    ratatui::widgets::Paragraph::new(text)
                        .wrap(ratatui::widgets::Wrap { trim: false })
                        .bold(),
                    block_area.inner(ratatui::layout::Margin::new(4, 2)),
                );
            })
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    ratatui::restore();
}
