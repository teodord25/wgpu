use std::ffi::OsStr;
use std::fs::{self, ReadDir};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};

use notify::event::ModifyKind;
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
    last_reload: Instant,
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
            last_reload: Instant::now() - Duration::from_secs(1), // in the past
            shader_watcher: watcher,
        }
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        // Drain FS events every loop tick
        while let Ok(ev) = self.shader_rx.try_recv() {
            // only care about Modify(Data(_)) events
            if let EventKind::Modify(ModifyKind::Data(_)) = ev.kind {
                let now = Instant::now();
                // debounce by 200ms
                if now.duration_since(self.last_reload) > Duration::from_millis(200) {
                    self.last_reload = now;

                    // confirm there's at least one .wgsl file
                    let has_shader = std::fs::read_dir("src/shaders")
                        .unwrap()
                        .filter_map(Result::ok)
                        .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("wgsl"));

                    if has_shader {
                        println!("🔄 hot-reloading shaders…");
                        self.gpu.as_mut().unwrap().reload_shader_pipeline();
                        self.window.as_ref().unwrap().request_redraw();
                    }
                }
            }
        }

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

                _ => {}
            }
        }
    }
}
