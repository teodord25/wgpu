use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub time: f32,           // 4 bytes  @ offset 0
    _pad0: f32,              // 4 bytes  @ offset 4 (align next field at 8)
    pub center: [f32; 2],    // 8 bytes  @ offset 8
    pub zoom: f32,           // 4 bytes  @ offset 16
    _pad1: f32,              // 4 bytes  @ offset 20
} // total size = 24 bytes

impl Uniforms {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            _pad0: 0.0,
            center: [-0.0, -1.0],
            zoom: 1.0,
            _pad1: 0.0,
        }
    }
}
