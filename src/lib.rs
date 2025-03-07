//! The `tui-shader` crate enables GPU accelerated styling for [`Ratatui`](https://ratatui.rs)
//! based applications.
//!
//! Computing styles at runtime can be expensive when run on the CPU, despite the
//! small "resolution" of cells in a terminal window. Utilizing the power of the
//! GPU helps us update the styling in the terminal at considerably higher framerates.
//!
//! ## Quickstart
//!
//! Add `ratatui` and `tui-shader` as dependencies to your Corgo.toml:
//!
//! ```shell
//! cargo add ratatui tui-shader
//! ```
//!
//! Then create a new application:
//!
//! ```rust,no_run
//! # use tui_shader::{ShaderCanvas, ShaderCanvasState};
//! let mut terminal = ratatui::init();
//! let mut state = ShaderCanvasState::default();
//! while state.get_instant().elapsed().as_secs() < 7 {
//!     terminal.draw(|frame| {
//!         frame.render_stateful_widget(ShaderCanvas::new(),
//!             frame.area(),
//!             &mut state);
//!     }).unwrap();
//! }
//! ratatui::restore();
//! ```
//!
//! And run it
//! ```shell
//! cargo run
//! ```
//!
//! Well that was lame. Where are all the cool shader-y effects?
//! We haven't actually provided a shader that the application should use so our [`ShaderCanvasState`]
//! uses a default shader, which always returns the magenta color. This happends because we created it
//! using [`ShaderCanvasState::default()`]. Now, let's write a `wgsl` shader and render some fancy stuff
//! in the terminal.
//!
//! ```wgsl
//! struct Context {
//!    time: f32,
//! }
//!
//! @group(0) @binding(0) var<uniform> input: Context;
//!
//! @fragment
//! fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
//!     let x = sin(1.0 - uv.x + input.time);
//!     let y = cos(1.0 - uv.y + input.time);
//!
//!     let b = 0.5;

//!     let d = 1.0 - distance(vec2<f32>(0.5), uv);
//!     let color = vec4<f32>(r, g, b, 1.0) * d;
//!     return color;
//! }
//! ```
//!
//! Ok, there's a lot to `unwrap` here. At first we define a `struct` that has a `time` field of type `f32`.
//! The `time` field is filled with data out-of-the-box by our [`ShaderCanvasState`] and can be used in any shader.
//! The important thing to note is that it isn't the name of the field that's important, it's the position in
//! the struct that determines which data we are reading from it.
//!
//! Next, we declare a variable `input` of type `Context` which reads the data at `@group(0) @binding(0)`.
//!
//! Finally - and this is were the magic happens - we can define our function for
//! manipulating colors. We must denote our function with `@fragment` because we are writing a fragment shader. As
//! long as we only define a single `@fragment` function in our file, we can name it whatever we want. Otherwise, we
//! must create our [`ShaderCanvasState`] using [`ShaderCanvasState::new_with_entry_point`] and pass in the name of the desired `@fragment` function.
//! A vertex shader cannot be provided as is always uses a single triangle, full-screen vertex
//! shader.
//!
//! We can use the UV coordinates provided by the vertex shader with `@location(0) uv: vec2<f32>`.
//! Now we have time and UV coordinates to work with to create amazing shaders. This shader just
//! does some math with these values and returns a new color. Time to get creative!
//!
//! Now, all we need to do is create our [`ShaderCanvasState`] using [`ShaderCanvasState::new`] and pass in our shader.
//!
//! ```rust,no_run
//! # use tui_shader::{ShaderCanvas, ShaderCanvasState, WgslShader};
//! let mut terminal = ratatui::init();
//! let shader = WgslShader::Path("shader.wgsl");
//! let mut state = ShaderCanvasState::new(shader);
//! while state.get_instant().elapsed().as_secs() < 5 {
//!     terminal.draw(|frame| {
//!         frame.render_stateful_widget(ShaderCanvas::new(),
//!             frame.area(),
//!             &mut state);
//!     }).unwrap();
//! }
//! ratatui::restore();
//! ```
//!
//! Now that's more like it!

mod canvas;
mod context;
mod state;
mod style;
mod util;

pub use crate::canvas::*;
pub use crate::state::*;
pub use crate::style::*;
pub use crate::util::*;

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, layout::Position};

    use crate::{CharacterRule, ShaderCanvas, ShaderCanvasState, context::ShaderContext};

    #[test]
    fn default_state() {
        let mut state = ShaderCanvasState::default();
        let raw_buffer = state.execute(ShaderContext::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 255, 255]));
    }

    #[test]
    fn different_entry_points() {
        let mut state = ShaderCanvasState::new_with_entry_point(
            wgpu::include_wgsl!("shaders/test_fragment.wgsl"),
            "green",
        );
        let raw_buffer = state.execute(ShaderContext::default());
        assert!(raw_buffer.iter().all(|pixel| pixel == &[0, 255, 0, 255]));
    }

    #[test]
    fn character_rule_map() {
        let mut terminal = ratatui::Terminal::new(TestBackend::new(64, 64)).unwrap();
        let mut state = ShaderCanvasState::default();
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(
                    ShaderCanvas::new().character_rule(CharacterRule::Map(|sample| {
                        if sample.x() == 0 { ' ' } else { '.' }
                    })),
                    frame.area(),
                    &mut state,
                );
                let buffer = frame.buffer_mut();
                for x in 0..buffer.area.width {
                    for y in 0..buffer.area.height {
                        if x == 0 {
                            assert_eq!(buffer.cell_mut(Position::new(x, y)).unwrap().symbol(), " ");
                        } else {
                            assert_eq!(buffer.cell_mut(Position::new(x, y)).unwrap().symbol(), ".");
                        }
                    }
                }
            })
            .unwrap();
        ratatui::restore();
    }
}
