pub mod shader_canvas;
mod wgpu_context;

#[cfg(test)]
mod tests {
    use wgpu_context::WgpuContext;

    use super::*;

    #[test]
    fn all_magenta() {
        let mut context = WgpuContext::new("src/shaders/default_fragment.wgsl");
        let raw_buffer = context.execute(64, 64);
        assert!(raw_buffer.iter().all(|pixel| pixel == &[255, 0, 255, 255]));
    }
}
