use crate::{Pixel, ShaderInput};

use super::{NoUserData, TuiShaderBackend};

pub struct CpuBackend<T> {
    callback: Box<dyn CpuShaderCallback<T>>,
}

impl CpuBackend<NoUserData> {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(u32, u32) -> Pixel + 'static,
    {
        Self {
            callback: Box::new(CpuShaderCallbackWithoutUserData(callback)),
        }
    }
}

impl<T> CpuBackend<T> {
    pub fn new_with_user_data<F>(callback: F) -> Self
    where
        F: Fn(u32, u32, &T) -> Pixel + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl<T> TuiShaderBackend<T> for CpuBackend<T> {
    fn execute(&mut self, shader_input: &ShaderInput, user_data: &T) -> Vec<Pixel> {
        let width = shader_input.resolution[0];
        let height = shader_input.resolution[1];
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let value = self.callback.call(x, y, user_data);
                pixels.push(value);
            }
        }
        pixels
    }
}

pub trait CpuShaderCallback<T> {
    fn call(&self, x: u32, y: u32, user_data: &T) -> Pixel;
}

impl<T, F> CpuShaderCallback<T> for F
where
    F: Fn(u32, u32, &T) -> Pixel,
{
    fn call(&self, x: u32, y: u32, user_data: &T) -> Pixel {
        self(x, y, user_data)
    }
}

// NewType required to avoid conflicting implementations
struct CpuShaderCallbackWithoutUserData<F>(F);
impl<F> CpuShaderCallback<NoUserData> for CpuShaderCallbackWithoutUserData<F>
where
    F: Fn(u32, u32) -> Pixel,
{
    fn call(&self, x: u32, y: u32, _user_data: &NoUserData) -> Pixel {
        self.0(x, y)
    }
}
