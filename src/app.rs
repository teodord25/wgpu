use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::gpu;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,

    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::Init = cause {
            let attrs = Window::default_attributes()
                .with_inner_size(Size::Physical(PhysicalSize::new(800, 400)))
                .with_visible(true);
            let window = Arc::new(event_loop.create_window(attrs).unwrap());
            self.window = Some(window);
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(Size::Physical(PhysicalSize::new(800, 400)))
                        .with_visible(true),
                )
                .unwrap(),
        );

        self.window = Some(window);

        let window = self.window.as_ref().unwrap();
        (
            self.config,
            self.surface,
            self.render_pipeline,
            self.vertex_buffer,
            self.queue,
            self.device,
        ) = gpu::create_gpu_state(&window);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // 1) grab wgpu state
                let surface = self.surface.as_ref().unwrap();
                let device = self.device.as_ref().unwrap();
                let queue = self.queue.as_ref().unwrap();
                let config = self.config.as_ref().unwrap();
                let pipeline = self.render_pipeline.as_ref().unwrap();
                let vertex_buffer = self.vertex_buffer.as_ref().unwrap();

                // 2) acquire next frame
                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&Default::default());

                // 3) encode a render pass that clears green and draws the quad
                let mut encoder = device.create_command_encoder(&Default::default());
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
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                    rpass.set_pipeline(pipeline);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.draw(0..4, 0..1);
                }

                // 4) submit + present
                queue.submit(Some(encoder.finish()));
                frame.present();

                // 5) schedule next frame (for continuous rendering)
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
