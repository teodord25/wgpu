use std::{fs, num::NonZeroU64};
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use wgpu::StoreOp;
use winit::window::Window;
use glam::{Mat4, Vec3};

use crate::vertex;

pub struct RenderResources {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    camera_buffer: wgpu::Buffer,
    model_buffer:  wgpu::Buffer,
    light_buffer:  wgpu::Buffer,
    ubo_bind_group: wgpu::BindGroup,

    start_time: Instant,

    pub dragging: bool,
    pub last_mouse_pos: (f32, f32),

    pub depth_view: wgpu::TextureView,
}

fn create_depth_view(
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

pub fn create_gpu_state(window: &Arc<Window>) -> RenderResources {
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

    // 1️⃣ Camera UBO: view_proj matrix (4×4 f32 = 64 bytes)
    let aspect = config.width as f32 / config.height as f32;
    let proj   = Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);
    let view   = Mat4::look_at_rh(Vec3::new(3.,2.,4.), Vec3::ZERO, Vec3::Y);
    let view_proj: [[f32;4];4] = (proj * view).to_cols_array_2d();

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("Camera UBO"),
        contents: bytemuck::cast_slice(&view_proj),  // &[ [f32;4];4 ]
        usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 2️⃣ Model UBO: identity matrix to start (also 64 bytes)
    let model_mat: [[f32;4];4] = Mat4::IDENTITY.to_cols_array_2d();
    let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("Model UBO"),
        contents: bytemuck::cast_slice(&model_mat),  // &[ [f32;4];4 ]
        usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 3️⃣ Light UBO: two vec4s (direction.xyz+pad, color.xyz+pad) = 32 bytes
    // Here we pack dir.xyz into [f32;4] (last component unused), same for color.
    let light_dir_color: [[f32;4];2] = [
        [ 0.0, -1.0, -1.0, 0.0 ],  // light direction
        [ 1.0,  1.0,  1.0, 0.0 ],  // light color
    ];
    let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("Light UBO"),
        contents: bytemuck::cast_slice(&light_dir_color),  // &[ [f32;4];2 ]
        usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UBO Bind Group Layout"),
            entries: &[
                // binding 0 = Camera UBO (mat4x4)
                wgpu::BindGroupLayoutEntry {
                    binding:    0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty:                wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:  Some( NonZeroU64::new(64).unwrap() ), // 4×4 f32
                    },
                    count: None,
                },
                // binding 1 = Model UBO (mat4x4)
                wgpu::BindGroupLayoutEntry {
                    binding:    1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty:                wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:  Some( NonZeroU64::new(64).unwrap() ),
                    },
                    count: None,
                },
                // binding 2 = Light UBO (vec3 + padding)
                wgpu::BindGroupLayoutEntry {
                    binding:    2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty:                wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:  Some( NonZeroU64::new(32).unwrap() ), // vec3 + pad
                    },
                    count: None,
                },
            ],
    });


    // 2.1 Camera UBO
    let aspect = config.width as f32 / config.height as f32;
    let proj   = Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);
    let view   = Mat4::look_at_rh(Vec3::new(3.,2.,4.), Vec3::ZERO, Vec3::Y);
    let view_proj: [[f32;4];4] = (proj * view).to_cols_array_2d();

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera UBO"),
        contents: bytemuck::cast_slice(&view_proj),
        usage:  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 2.2 Model UBO (we’ll rotate around Y)
    let model_mat: [[f32;4];4] = Mat4::IDENTITY.to_cols_array_2d();
    let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Model UBO"),
        contents: bytemuck::cast_slice(&model_mat),
        usage:  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 2.3 Light UBO
    // direction + color, pad to 16 bytes
    let light_data: [f32;4] = [ 0.0, -1.0, -1.0, 0.0 ]; // dir.xyz + pad
    let light_color: [f32;4] = [ 1.0, 1.0, 1.0, 0.0 ];
    let mut light_buf_data = Vec::new();
    light_buf_data.extend_from_slice(&light_data);
    light_buf_data.extend_from_slice(&light_color);

    let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light UBO"),
        contents: &bytemuck::cast_slice(&light_buf_data),
        usage:  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 2.4 Single bind group with 3 entries
    let ubo_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: model_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: light_buffer.as_entire_binding(),
            },
        ],
        label: Some("UBO Bind Group"),
    });

    let pipeline = create_pipeline(&device, &config, &uniform_bind_group_layout);
    let depth_view = create_depth_view(&device, &config);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cube Vertex Buffer"),
        contents: bytemuck::cast_slice(vertex::VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cube Index Buffer"),
        contents: bytemuck::cast_slice(vertex::INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let num_indices = vertex::INDICES.len() as u32;

    RenderResources {
        surface,
        device,
        queue,
        config,
        pipeline,

        vertex_buffer,
        index_buffer,
        num_indices,

        camera_buffer,
        model_buffer,
        light_buffer,
        ubo_bind_group,

        start_time: std::time::Instant::now(),

        dragging: false,
        last_mouse_pos: (0.0, 0.0),

        depth_view,
    }
}

// Builds a very basic render pipeline that draws in solid green
fn create_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    uniform_bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::RenderPipeline {

    let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Cube VS"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/cube.vert.wgsl").into()),
    });
    let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Cube FS"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/cube.frag.wgsl").into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[uniform_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            compilation_options: Default::default(),
            module: &vs_module,
            entry_point: Some("vs_main"),
            buffers: &[vertex::Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            compilation_options: Default::default(),
            module: &fs_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // passes if new depth < old
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
    })
}

fn create_pipeline_with_shader(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    vs_shader: &wgpu::ShaderModule,
    fs_shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[uniform_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            compilation_options: Default::default(),
            module: &vs_shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex::Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            compilation_options: Default::default(),
            module: &fs_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // passes if new depth < old
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
    })
}

impl RenderResources {
    pub fn reload_shader_pipeline(&mut self) {

        let vs_src = fs::read_to_string("src/shaders/cube.vert.wgsl")
            .expect("Failed to re-read vertex shader");
        let fs_src = fs::read_to_string("src/shaders/cube.frag.wgsl")
            .expect("Failed to re-read fragment shader");

        let vs_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cube VS"),
            source: wgpu::ShaderSource::Wgsl(vs_src.into()),
        });
        let fs_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cube FS"),
            source: wgpu::ShaderSource::Wgsl(fs_src.into()),
        });

        let uniform_bind_group_layout =
            self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("UBO Bind Group Layout"),
                entries: &[
                    // binding 0 = Camera UBO (mat4x4)
                    wgpu::BindGroupLayoutEntry {
                        binding:    0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty:                wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size:  Some( NonZeroU64::new(64).unwrap() ), // 4×4 f32
                        },
                        count: None,
                    },
                    // binding 1 = Model UBO (mat4x4)
                    wgpu::BindGroupLayoutEntry {
                        binding:    1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty:                wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size:  Some( NonZeroU64::new(64).unwrap() ),
                        },
                        count: None,
                    },
                    // binding 2 = Light UBO (vec3 + padding)
                    wgpu::BindGroupLayoutEntry {
                        binding:    2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty:                wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size:  Some( NonZeroU64::new(32).unwrap() ), // vec3 + pad
                        },
                        count: None,
                    },
                ],
        });

        let pipeline = create_pipeline_with_shader(&self.device, &self.config, &uniform_bind_group_layout, &vs_module, &fs_module);
        self.pipeline = pipeline;

        println!("✅ shader pipeline reloaded");
    }

    pub fn resolution(&self) -> (f32, f32) {
        (self.config.width as f32, self.config.height as f32)
    }

    pub fn render(&mut self, window: &Option<Arc<Window>>) {
        // 1) state already ready

        // 2) acquire next frame
        let window = window.as_ref().unwrap();
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        // 3) encode a render pass that clears green and draws the quad
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),

                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            rpass.set_bind_group(0, &self.ubo_bind_group, &[]);

            rpass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        let t = self.start_time.elapsed().as_secs_f32();
        let model_rot: [[f32;4];4] = Mat4::from_rotation_y(t).to_cols_array_2d();
        self.queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&model_rot));

        // 4) submit + present
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        // 5) schedule next frame (for continuous rendering)
        window.as_ref().request_redraw();
    }
}
