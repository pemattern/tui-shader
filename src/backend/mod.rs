pub mod cpu;
pub mod wgpu;

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NoUserData(f32);

pub trait TuiShaderBackend<T> {
    fn execute(&mut self, width: u16, height: u16, user_data: &T) -> Vec<[u8; 4]>;
}
