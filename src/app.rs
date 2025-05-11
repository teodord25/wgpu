use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::event::{MouseScrollDelta, ElementState, MouseButton};

use crate::gpu;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<gpu::RenderResources>,
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::Init = cause {
            let attrs = Window::default_attributes()
                .with_inner_size(Size::Physical(PhysicalSize::new(800, 400)))
                .with_visible(true);
            self.window = Some(Arc::new(event_loop.create_window(attrs).unwrap()));
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
        self.window = Some(window.clone());
        self.gpu = Some(gpu::create_gpu_state(&window));
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(gpu) = self.gpu.as_mut() {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::RedrawRequested => gpu.render(&self.window),

                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        gpu.dragging = state == ElementState::Pressed;
                        println!("Dragging: {}", gpu.dragging);
                    }
                },

                WindowEvent::MouseWheel { delta, .. } => {
                    println!("MouseWheel event: {:?}", delta);
                    let raw_scroll = match delta {
                        MouseScrollDelta::LineDelta(_, y)    => y,
                        MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 120.0, // normalize pixels to “line” units
                    };

                    let zoom_speed = 0.1;
                    let scale = 1.0 - raw_scroll * zoom_speed;

                    gpu.uniforms.zoom = (gpu.uniforms.zoom * scale).max(0.1);
                    println!("Updated zoom: {}", gpu.uniforms.zoom);

                    self.window.as_ref().unwrap().request_redraw();
                }

                WindowEvent::CursorMoved { position, .. } => {
                    println!("Cursor moved: {:?}", position);
                    let (x, y) = (position.x as f32, position.y as f32);
                    if gpu.dragging {
                        let (width, height) = gpu.resolution();
                        let dx = (x - gpu.last_mouse_pos.0) / width / gpu.uniforms.zoom;
                        let dy = (y - gpu.last_mouse_pos.1) / height / gpu.uniforms.zoom;
                        gpu.uniforms.center[0] -= dx;
                        gpu.uniforms.center[1] -= dy;
                    }
                    gpu.last_mouse_pos = (x, y);
                }

                _ => {}
            }
        }
    }
}
