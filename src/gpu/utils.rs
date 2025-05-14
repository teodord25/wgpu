use std::{fs, ops::Deref};

pub fn create_depth_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let texture = device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub struct VertexShader(pub wgpu::ShaderModule);
pub struct FragmentShader(pub wgpu::ShaderModule);

impl Deref for VertexShader {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for FragmentShader {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn load_shader(label: &str, path: &str, device: &wgpu::Device) -> wgpu::ShaderModule {
    let src = fs::read_to_string(path).expect("failed to read shader file");
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(src.into()),
    })
}

