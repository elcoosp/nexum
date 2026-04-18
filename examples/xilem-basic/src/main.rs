use nexum_core::{Config, DeepLinkHandle};
use nexum_xilem::{setup, with_deep_links};
use winit::event_loop::EventLoop;
use xilem::view::{flex_col, label, CrossAxisAlignment, MainAxisAlignment};
use xilem::{WidgetView, WindowOptions, Xilem};

struct AppState {
    handle: DeepLinkHandle,
    last_url: Option<String>,
}

fn app_view(state: &mut AppState) -> impl WidgetView<AppState> {
    let url_text = match &state.last_url {
        Some(url) => format!("Received: {}", url),
        None => "Waiting for deep link...".to_string(),
    };

    let main_view = flex_col((label("Hello, deep links!"), label(url_text)))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::Center);

    with_deep_links(
        main_view,
        state.handle.clone(),
        |state: &mut AppState, url| {
            // <-- Added type annotation here
            state.last_url = Some(url);
        },
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = setup(Config {
        schemes: vec!["xilem".to_string()],
        app_links: vec![],
    });

    let state = AppState {
        handle,
        last_url: None,
    };

    let app = Xilem::new_simple(state, app_view, WindowOptions::new("Deep Link Tester"));
    app.run_in(EventLoop::with_user_event())?;

    Ok(())
}
