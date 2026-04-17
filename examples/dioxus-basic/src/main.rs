use dioxus::prelude::*;
use nexum_core::Config;
use nexum_dioxus::{use_deep_link_listener, DEEP_LINK_URLS};

fn App() -> Element {
    let config = Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };
    use_deep_link_listener(config);

    let urls = DEEP_LINK_URLS.read();

    rsx! {
        div {
            h1 { "Hello, deep links!" }
            p {
                match urls.as_ref() {
                    Some(urls) => format!("Last URL: {:?}", urls),
                    None => "No deep link received".to_string(),
                }
            }
        }
    }
}

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new().with_title("Nexum Dioxus Example"),
            ),
        )
        .launch(App);
}
