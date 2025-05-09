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
                _ => {}
            }
        }
    }
}
