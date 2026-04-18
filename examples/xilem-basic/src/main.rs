use nexum_core::Config;
use nexum_xilem::setup;
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoop;
use xilem::core::fork;
use xilem::view::{flex_col, label, task_raw, CrossAxisAlignment, MainAxisAlignment};
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
    // 1. Initialize deep‑link system and get the handle.
    let handle = setup(Config {
        schemes: vec!["xilem".to_string()],
        app_links: vec![],
    });

    // 2. Create shared state.
    let state = AppState::new();

    // 3. Define the app logic with a background task that awaits URLs.
    fn app_with_task(
        state: &mut AppState,
        handle: nexum_core::DeepLinkHandle,
    ) -> impl WidgetView<AppState> {
        let main_view = app_view(state);
        let _state_clone = state.clone(); // Not used, but kept for clarity.
        let handle_clone = handle.clone(); // Clone to move into the task closure.

        let url_task = task_raw(
            move |proxy| {
                let handle = handle_clone.clone();
                async move {
                    while let Some(url) = handle.recv().await {
                        if proxy.message(url).is_err() {
                            break;
                        }
                    }
                }
            },
            move |state: &mut AppState, url: String| {
                state.set_url(url);
            },
        );

        fork(main_view, url_task)
    }

    // 4. Run the Xilem app.
    let app = Xilem::new_simple(
        state,
        move |state| app_with_task(state, handle.clone()),
        WindowOptions::new("Deep Link Tester"),
    );
    app.run_in(EventLoop::with_user_event())?;

    Ok(())
}
