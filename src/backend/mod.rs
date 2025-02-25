pub mod wgpu;

pub trait TuiShaderBackend: Eq {
    fn execute(&mut self, width: u16, height: u16) -> Vec<[u8; 4]>;
}
