#[repr(C)]
#[derive(Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ShaderContext {
    // struct field order matters
    time: f32,
    padding: f32,
    resolution: [u32; 2],
}

impl ShaderContext {
    pub(crate) fn new(time: f32, width: u16, height: u16) -> Self {
        Self {
            time,
            padding: f32::default(),
            resolution: [width.into(), height.into()],
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.resolution[0]
    }

    pub(crate) fn height(&self) -> u32 {
        self.resolution[1]
    }
}

impl Default for ShaderContext {
    fn default() -> Self {
        Self {
            time: 0.0,
            padding: 0.0,
            resolution: [64, 64],
        }
    }
}
