use std::error::Error;

use wgpu::naga;

/// Utility `enum` to pass in a shader into [`ShaderCanvasState`](crate::ShaderCanvasState). Another option is to use the re-exported
/// [`include_wgsl!`](wgpu::include_wgsl!) macro, which checks at runtime if the path to the file is valid and returns a
/// [`ShaderModuleDescriptor`](wgpu::ShaderModuleDescriptor).
pub enum WgslShader<'a> {
    /// Use wgsl source code in a `&str`.
    Source(&'a str),

    /// Use a path to a wgsl shader.
    Path(&'a str),
}

impl<'a> TryFrom<WgslShader<'a>> for wgpu::ShaderModuleDescriptor<'a> {
    type Error = Box<dyn Error>;
    fn try_from(value: WgslShader<'a>) -> Result<wgpu::ShaderModuleDescriptor<'a>, Self::Error> {
        match value {
            WgslShader::Source(source) => create_shader_module_descriptor(source.to_string()),
            WgslShader::Path(path) => {
                let source = match std::fs::read_to_string(path) {
                    Ok(source) => source,
                    Err(error) => return Err(Box::new(error)),
                };
                create_shader_module_descriptor(source)
            }
        }
    }
}

fn create_shader_module_descriptor<'a>(
    source: String,
) -> Result<wgpu::ShaderModuleDescriptor<'a>, Box<dyn Error>> {
    match naga::front::wgsl::parse_str(source.as_str()) {
        Ok(_) => Ok(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        }),
        Err(error) => Err(Box::new(error)),
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
