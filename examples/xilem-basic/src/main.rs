use nexum_core::Config;
use nexum_xilem::{setup, with_deep_links};
use xilem::winit::event_loop::EventLoop;
use xilem::{
    view::{flex_col, label},
    WidgetView,
    Xilem, // Use Xilem instead of App
};

#[derive(Default)]
struct AppState {
    last_url: Option<String>,
}

// The app logic function now returns the wrapped view directly
fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    // Use flex_col for a vertical stack
    let content = flex_col((
        label("Hello, deep links!"),
        label(
            state
                .last_url
                .clone()
                .unwrap_or_else(|| "No deep link received".to_string()),
        ),
    ));

    // Wrap the content with deep link handling
    with_deep_links(content, |state: &mut AppState, url| {
        state.last_url = Some(url);
    })
}

fn main() {
    let config = Config {
        schemes: vec!["xilem".to_string()],
        app_links: vec![],
    };
    setup(config);

    // Create the Xilem app with state and logic, then run it
    Xilem::new_simple(
        AppState::default(),
        app_logic,
        xilem::WindowOptions::new("Deep Link Demo"),
    )
    .run_in(EventLoop::with_user_event())
    .unwrap();
}
