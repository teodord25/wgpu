use std::sync::Arc;
use winit::window::Window;

use crate::vertex;

pub fn create_gpu_state(
    window: &Arc<Window>,
) -> (
    Option<wgpu::SurfaceConfiguration>,
    Option<wgpu::Surface<'static>>,
    Option<wgpu::RenderPipeline>,
    Option<wgpu::Buffer>,
    Option<wgpu::Queue>,
    Option<wgpu::Device>,
) {
    // create wgpu instance and surface
    let instance = wgpu::Instance::default();

    let raw = instance.create_surface(window).unwrap();
    let surface = unsafe { std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(raw) };

    // choose an adapter
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("Failed to find an appropriate adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        },
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
    }))
    .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats[0]; // choose a supported format?

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    return (
        Some(config.clone()),
        Some(surface),
        Some(create_pipeline(&device, &config)),
        Some(vertex::create_quad_vertex_buffer(&device)),
        Some(queue),
        Some(device),
    );
}

// Builds a very basic render pipeline that draws in solid green
fn create_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Basic Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            compilation_options: Default::default(),
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex::Vertex::LAYOUT],
        },
        fragment: Some(wgpu::FragmentState {
            compilation_options: Default::default(),
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
    })
}
