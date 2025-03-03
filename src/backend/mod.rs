use crate::{Pixel, ShaderContext};

pub mod cpu;
pub mod wgpu;

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NoUserData(f32);

pub trait TuiShaderBackend<T> {
    fn execute(&mut self, ctx: ShaderContext, user_data: &T) -> Vec<Pixel>;
}
