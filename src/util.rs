/// Utility `enum` to pass in a shader into [`ShaderCanvasState`](crate::ShaderCanvasState). Another option is to use the re-exported
/// [`include_wgsl!`](wgpu::include_wgsl!) macro, which checks at runtime if the path to the file is valid and returns a
/// [`ShaderModuleDescriptor`](wgpu::ShaderModuleDescriptor).
pub enum WgslShader<'a> {
    /// Use wgsl source code in a `&str`.
    Source(&'a str),

    /// Use a path to a wgsl shader.
    Path(&'a str),
}

impl<'a> From<WgslShader<'a>> for wgpu::ShaderModuleDescriptor<'a> {
    fn from(value: WgslShader<'a>) -> Self {
        match value {
            WgslShader::Source(source) => wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            },
            WgslShader::Path(path) => {
                let source = std::fs::read_to_string(path).expect("unable to read file");
                wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                }
            }
        }
    }
}

pub(crate) type Pixel = [u8; 4];

pub(crate) fn bytes_per_row(width: u32) -> u32 {
    let row_size = width * 4;
    (row_size + 255) & !255
}

pub(crate) fn row_padding(width: u32) -> u32 {
    let row_size = width * 4;
    let bytes_per_row = bytes_per_row(width);
    (bytes_per_row - row_size) / 4
}
