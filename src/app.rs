use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;

use notify::{Event, EventKind, RecommendedWatcher, Watcher};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::event::{MouseScrollDelta, ElementState, MouseButton};

use crate::gpu;

pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<gpu::RenderResources>,

    shader_rx: Receiver<Event>,
    #[allow(dead_code)]
    shader_watcher: RecommendedWatcher, // keep it alive
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            })
            .expect("watcher init failed");

        let path = Path::new("src/shaders");
        watcher
            .watch(path, notify::RecursiveMode::NonRecursive)
            .expect("watch failed");

        App {
            window: None,
            gpu: None,

            shader_rx: rx,
            shader_watcher: watcher,
        }
    }
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
        while let Ok(ev) = self.shader_rx.try_recv() {
            // does this event touch shader.wgsl?
            if ev.paths.iter().any(|p| p.ends_with("shader.wgsl")) {
                match ev.kind {
                    EventKind::Modify(_)
                  | EventKind::Create(_)
                  | EventKind::Remove(_) => { 
                      if std::path::Path::new("src/shaders/shader.wgsl").exists() {
                          println!("🔄 shader changed ({:?}), reloading…", ev.kind);
                          self.gpu.as_mut().unwrap().reload_shader_pipeline();
                      } else {
                          println!("⚠️ shader file missing, skipping reload");
                      }
                  },
                    _ => {},
                }
            }
        }

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
