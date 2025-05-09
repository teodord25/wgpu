use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub time: f32,
}

impl Uniforms {
    pub fn new() -> Self {
        Self { time: 0.0 }
    }
}
