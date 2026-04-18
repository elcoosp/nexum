use nexum_xilem::{create_deep_link_receiver, register_delegate};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use winit::event_loop::EventLoop;
use xilem::core::fork;
use xilem::view::{flex_col, label, task, CrossAxisAlignment, MainAxisAlignment};
use xilem::{WidgetView, WindowOptions, Xilem};

#[derive(Clone)]
struct AppState {
    last_url: Arc<Mutex<Option<String>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            last_url: Arc::new(Mutex::new(None)),
        }
    }

    fn get_url(&self) -> Option<String> {
        self.last_url.lock().unwrap().clone()
    }

    fn set_url(&self, url: String) {
        *self.last_url.lock().unwrap() = Some(url);
    }
}

fn app_view(state: &mut AppState) -> impl WidgetView<AppState> {
    let url_text = match state.get_url() {
        Some(url) => format!("Received: {}", url),
        None => "Waiting for deep link...".to_string(),
    };

    flex_col((label("Hello, deep links!"), label(url_text)))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::Center)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Deep-link receiver thread
    let rx = create_deep_link_receiver();
    let state = AppState::new();
    let state_clone = state.clone();

    thread::spawn(move || {
        while let Ok(url) = rx.recv_blocking() {
            state_clone.set_url(url);
        }
    });

    // 2. Register macOS Apple Event handler
    register_delegate();

    // 3. Polling task to trigger rebuilds
    fn app_with_polling(state: &mut AppState) -> impl WidgetView<AppState> {
        let main_view = app_view(state);
        let poll_task = task(
            move |proxy| async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                loop {
                    interval.tick().await;
                    if proxy.message(()).is_err() {
                        break;
                    }
                }
            },
            |_: &mut AppState, ()| {},
        );
        fork(main_view, poll_task)
    }

    // 4. Run the app
    let app = Xilem::new_simple(
        state,
        app_with_polling,
        WindowOptions::new("Deep Link Tester"),
    );
    app.run_in(EventLoop::with_user_event())?;

    Ok(())
}
