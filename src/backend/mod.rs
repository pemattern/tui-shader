pub mod wgpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NoUserData;

pub trait TuiShaderBackend<T>: Eq {
    fn execute(&mut self, width: u16, height: u16) -> Vec<[u8; 4]>;
    fn set_user_data(&mut self, user_data: T);
}
