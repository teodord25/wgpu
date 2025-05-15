use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
    pub uv:       [f32; 2], // ← new
}
impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![
            0 => Float32x3,  // position
            1 => Float32x3,  // normal
            2 => Float32x2   // uv
        ];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as _,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &Self::ATTRIBS,
        }
    }
}

// 24 vertices (4 per face) so each face can have its own normal and UV coords
pub const VERTICES: &[Vertex] = &[
    // +X face
    Vertex { position: [ 1., -1., -1.], normal: [1., 0., 0.], uv: [0.0, 0.0] },
    Vertex { position: [ 1.,  1., -1.], normal: [1., 0., 0.], uv: [0.0, 1.0] },
    Vertex { position: [ 1.,  1.,  1.], normal: [1., 0., 0.], uv: [1.0, 1.0] },
    Vertex { position: [ 1., -1.,  1.], normal: [1., 0., 0.], uv: [1.0, 0.0] },

    // -X face
    Vertex { position: [-1., -1.,  1.], normal: [-1., 0., 0.], uv: [0.0, 0.0] },
    Vertex { position: [-1.,  1.,  1.], normal: [-1., 0., 0.], uv: [0.0, 1.0] },
    Vertex { position: [-1.,  1., -1.], normal: [-1., 0., 0.], uv: [1.0, 1.0] },
    Vertex { position: [-1., -1., -1.], normal: [-1., 0., 0.], uv: [1.0, 0.0] },

    // +Y face
    Vertex { position: [-1.,  1., -1.], normal: [0., 1., 0.], uv: [0.0, 0.0] },
    Vertex { position: [-1.,  1.,  1.], normal: [0., 1., 0.], uv: [0.0, 1.0] },
    Vertex { position: [ 1.,  1.,  1.], normal: [0., 1., 0.], uv: [1.0, 1.0] },
    Vertex { position: [ 1.,  1., -1.], normal: [0., 1., 0.], uv: [1.0, 0.0] },

    // -Y face
    Vertex { position: [-1., -1.,  1.], normal: [0., -1., 0.], uv: [0.0, 0.0] },
    Vertex { position: [-1., -1., -1.], normal: [0., -1., 0.], uv: [0.0, 1.0] },
    Vertex { position: [ 1., -1., -1.], normal: [0., -1., 0.], uv: [1.0, 1.0] },
    Vertex { position: [ 1., -1.,  1.], normal: [0., -1., 0.], uv: [1.0, 0.0] },

    // +Z face
    Vertex { position: [-1., -1.,  1.], normal: [0., 0., 1.], uv: [0.0, 0.0] },
    Vertex { position: [ 1., -1.,  1.], normal: [0., 0., 1.], uv: [1.0, 0.0] },
    Vertex { position: [ 1.,  1.,  1.], normal: [0., 0., 1.], uv: [1.0, 1.0] },
    Vertex { position: [-1.,  1.,  1.], normal: [0., 0., 1.], uv: [0.0, 1.0] },

    // -Z face
    Vertex { position: [ 1., -1., -1.], normal: [0., 0., -1.], uv: [0.0, 0.0] },
    Vertex { position: [-1., -1., -1.], normal: [0., 0., -1.], uv: [1.0, 0.0] },
    Vertex { position: [-1.,  1., -1.], normal: [0., 0., -1.], uv: [1.0, 1.0] },
    Vertex { position: [ 1.,  1., -1.], normal: [0., 0., -1.], uv: [0.0, 1.0] },
];

// 6 faces × 2 triangles × 3 indices = 36
pub const INDICES: &[u16] = &[
     0,  1,  2,  0,  2,  3,   // +X
     4,  5,  6,  4,  6,  7,   // -X
     8,  9, 10,  8, 10, 11,   // +Y
    12, 13, 14, 12, 14, 15,   // -Y
    16, 17, 18, 16, 18, 19,   // +Z
    20, 21, 22, 20, 22, 23,   // -Z
];
