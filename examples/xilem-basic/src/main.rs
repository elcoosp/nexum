use url::Url;
use xilem::{view::flex, App, AppLauncher, WidgetView};

use nexum_core::Config;

#[derive(Debug)]
enum AppMessage {
    DeepLink(Vec<Url>),
    Clear,
}

struct AppState {
    last_urls: Option<Vec<Url>>,
}

impl AppState {
    fn new() -> Self {
        Self { last_urls: None }
    }
}

fn app_logic(state: &mut AppState) -> impl WidgetView<AppMessage> {
    let config = Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };

    flex((
        xilem::view::label("Hello, deep links!".to_string()),
        xilem::view::label(match &state.last_urls {
            Some(urls) => format!("Last URL: {:?}", urls),
            None => "No deep link received".to_string(),
        }),
        xilem::view::button("Clear", |_| AppMessage::Clear),
        nexum_xilem::deep_link_task(config, AppMessage::DeepLink),
    ))
}

impl App for AppState {
    type Message = AppMessage;

    fn update(&mut self, message: Self::Message) {
        match message {
            AppMessage::DeepLink(urls) => self.last_urls = Some(urls),
            AppMessage::Clear => self.last_urls = None,
        }
    }

    fn view(&mut self) -> xilem::WidgetView<Self::Message> {
        app_logic(self)
    }
}

fn main() {
    AppLauncher::new(AppState::new()).run();
}
