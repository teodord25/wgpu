use winit::event_loop::{ControlFlow, EventLoop};

mod gpu;
mod app;
mod vertex;
mod uniform;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::default();
    let _ = event_loop.run_app(&mut app);
}
