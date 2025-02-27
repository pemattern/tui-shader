pub mod wgpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NoUserData;

pub trait TuiShaderBackend: Eq {
    fn execute<T>(&mut self, width: u16, height: u16, user_data: Option<T>) -> Vec<[u8; 4]>;
}
