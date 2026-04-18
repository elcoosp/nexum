use nexum_xilem::{create_deep_link_receiver, register_delegate};
use std::thread;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

fn main() {
    // 1. Create the deep‑link receiver (must be before event loop).
    let rx = create_deep_link_receiver();

    // 2. Spawn a thread to print received URLs.
    thread::spawn(move || {
        while let Ok(url) = rx.recv_blocking() {
            println!("[app] Deep link: {}", url);
        }
    });

    // 3. Create the winit event loop.
    let event_loop = EventLoop::new().unwrap();

    // 4. Register the macOS delegate (no‑op on other platforms).
    register_delegate();

    // 5. Run the application.
    event_loop.run_app(MyApp::default()).unwrap();
}

#[derive(Default)]
struct MyApp {
    window: Option<Box<dyn Window>>,
}

impl ApplicationHandler for MyApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attrs = WindowAttributes::default()
            .with_title("Deep Link Tester")
            .with_visible(true);
        self.window = Some(event_loop.create_window(window_attrs).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }
    }
}
