use crate::{Pixel, ShaderContext};

use super::{NoUserData, TuiShaderBackend};

pub struct CpuBackend<T = NoUserData> {
    callback: Box<dyn CpuShaderCallback<T>>,
}

impl CpuBackend {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(u32, u32, ShaderContext) -> Pixel + 'static,
    {
        Self {
            callback: Box::new(CpuShaderCallbackWithoutUserData(callback)),
        }
    }
}

impl<T> CpuBackend<T> {
    pub fn new_with_user_data<F>(callback: F) -> Self
    where
        F: Fn(u32, u32, ShaderContext, &T) -> Pixel + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl<T> TuiShaderBackend<T> for CpuBackend<T> {
    fn execute(&mut self, ctx: ShaderContext, user_data: &T) -> Vec<Pixel> {
        let width = ctx.resolution[0];
        let height = ctx.resolution[1];
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let value = self.callback.call(x, y, ctx, user_data);
                pixels.push(value);
            }
        }
        pixels
    }
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new(|_, _, _| [255, 0, 255, 255])
    }
}

pub trait CpuShaderCallback<T = NoUserData> {
    fn call(&self, x: u32, y: u32, ctx: ShaderContext, user_data: &T) -> Pixel;
}

impl<T, F> CpuShaderCallback<T> for F
where
    F: Fn(u32, u32, ShaderContext, &T) -> Pixel,
{
    fn call(&self, x: u32, y: u32, ctx: ShaderContext, user_data: &T) -> Pixel {
        self(x, y, ctx, user_data)
    }
}

// NewType required to avoid conflicting implementations
struct CpuShaderCallbackWithoutUserData<F>(F);
impl<F> CpuShaderCallback for CpuShaderCallbackWithoutUserData<F>
where
    F: Fn(u32, u32, ShaderContext) -> Pixel,
{
    fn call(&self, x: u32, y: u32, ctx: ShaderContext, _user_data: &NoUserData) -> Pixel {
        self.0(x, y, ctx)
    }
}
