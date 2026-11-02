use nexum_xilem::Config;
use nexum_xilem::{setup, with_deep_links};
use xilem::view::{flex_col, label};
use xilem::{WidgetView, Xilem};

#[derive(Default)]
struct AppState {
    last_url: Option<String>,
}

fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let content = flex_col((
        label("Hello, deep links!"),
        label(
            state
                .last_url
                .clone()
                .unwrap_or_else(|| "No deep link received".to_string()),
        ),
    ));

    with_deep_links(content, |state: &mut AppState, url| {
        state.last_url = Some(url);
    })
}

fn main() {
    let config = Config::builder()
        .scheme("xilem")
        .build()
        .expect("valid config");
    setup(&config).expect("deep link setup failed");

    Xilem::new_simple(
        AppState::default(),
        app_logic,
        xilem::WindowOptions::new("Deep Link Demo"),
    )
    .run_in(xilem::winit::event_loop::EventLoop::with_user_event())
    .expect("xilem app failed to run");
}
