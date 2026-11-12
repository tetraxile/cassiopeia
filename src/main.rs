use winit::event_loop::{ControlFlow, EventLoop};

use cassiopeia::App;

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
