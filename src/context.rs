#[repr(C)]
#[derive(Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ShaderContext {
    // time[0] = seconds
    // time[1] = seconds * 10
    // time[2] = sin of seconds
    // time[3] = cos of seconds
    pub(crate) time: [f32; 4],

    // rect[0] = x,
    // rect[1] = y,
    // rect[2] = width,
    // rect[3] = height,
    pub(crate) rect: [u32; 4],
}

impl ShaderContext {
    pub(crate) fn new(time: f32, rect: ratatui::layout::Rect) -> Self {
        Self {
            time: [time, time * 10.0, time.sin(), time.cos()],
            rect: [
                rect.x.into(),
                rect.y.into(),
                rect.width.into(),
                rect.height.into(),
            ],
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.rect[2]
    }

    pub(crate) fn height(&self) -> u32 {
        self.rect[3]
    }
}

impl Default for ShaderContext {
    fn default() -> Self {
        Self {
            time: [0.0, 0.0, 0.0, 1.0],
            rect: [0, 0, 64, 64],
        }
    }
}
