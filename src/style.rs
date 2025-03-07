use ratatui::style::{Color, Style};

use crate::Pixel;

/// Determines which character to use for Cell.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CharacterRule {
    /// [`CharacterRule::Always`] takes a single char and applies it to all Cells in the [`ShaderCanvas`].
    Always(char),

    /// [`CharacterRule::Map`] takes a function as an argument and allows you to map the input [`Sample`] to
    /// a character.
    ///
    /// For example, one might use the red channel from the shader ([Sample::r]) and map
    /// it to a different character depending on the value:
    /// ```rust,no_run
    /// # use tui_shader::{CharacterRule, ShaderCanvas, StyleRule};
    /// let char_map = CharacterRule::Map(|sample| {
    ///     if (sample.r() > 127) {
    ///         '@'
    ///     } else {
    ///         'o'
    ///     }
    /// });
    ///
    /// let canvas = ShaderCanvas::new()
    ///     .character_rule(char_map)
    ///     .style_rule(StyleRule::ColorFg);
    /// ```
    Map(fn(Sample) -> char),
}

impl Default for CharacterRule {
    /// Returns `Self::Always(' ')`
    fn default() -> Self {
        Self::Always(' ')
    }
}

/// Determines how to use the output of the fragment shader to style a Cell.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum StyleRule {
    /// [`StyleRule::ColorFg`] only applies the color from the shader to the foreground of the Cell.
    ColorFg,

    /// [`StyleRule::ColorBg`] only applies the color from the shader to the background of the Cell. This
    /// is the default value.
    #[default]
    ColorBg,

    /// [`StyleRule::Map`] takes a function as an argument and allows you to map the input [`Sample`] to
    /// a Style. For example, one might use the transparency value from the shader ([Sample::a]) and set
    /// a cutoff for switching between a normal and reversed [`Style`].
    ///
    /// ```rust,no_run
    /// # use tui_shader::{ShaderCanvas, StyleRule};
    /// # use ratatui::style::{Style, Stylize};
    /// let style_map = StyleRule::Map(|sample| {
    ///     let style = Style::new().bg(sample.color());
    ///     if (sample.u() > 0.5) {
    ///         style
    ///     } else {
    ///         style.reversed()
    ///     }
    /// });
    ///
    /// let canvas = ShaderCanvas::new()
    ///     .style_rule(style_map);
    /// ```
    Map(fn(Sample) -> Style),
}

/// Primarily used in [`CharacterRule::Map`] and [`StyleRule::Map`], it provides access to a cells color and position
/// allowing to map the output of the shader to more complex behaviour.
pub struct Sample {
    pixel: Pixel,
    position: (u16, u16),
    uv: (f32, f32),
}

impl Sample {
    pub(crate) fn new(pixel: Pixel, position: (u16, u16), uv: (f32, f32)) -> Self {
        Self {
            pixel,
            position,
            uv,
        }
    }

    pub fn color(&self) -> Color {
        Color::Rgb(self.r(), self.g(), self.b())
    }

    /// The red channel of the [`Sample`]
    pub fn r(&self) -> u8 {
        self.pixel[0]
    }

    /// The green channel of the [`Sample`]
    pub fn g(&self) -> u8 {
        self.pixel[1]
    }

    /// The blue channel of the [`Sample`]
    pub fn b(&self) -> u8 {
        self.pixel[2]
    }

    /// The alpha channel of the [`Sample`]
    pub fn a(&self) -> u8 {
        self.pixel[3]
    }

    /// The x coordinate of the [`Sample`]
    pub fn x(&self) -> u16 {
        self.position.0
    }

    /// The y coordinate of the [`Sample`]
    pub fn y(&self) -> u16 {
        self.position.1
    }

    /// The normalized x coordinate of the [`Sample`]
    pub fn u(&self) -> f32 {
        self.uv.0
    }

    /// The normalized y coordinate of the [`Sample`]
    pub fn v(&self) -> f32 {
        self.uv.1
    }
}
