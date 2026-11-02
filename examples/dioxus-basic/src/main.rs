use dioxus::prelude::*;
use nexum_dioxus::{
    Config, DesktopConfig, LogicalSize, WindowBuilder, launch_desktop, use_deep_link,
};

fn app() -> Element {
    let deep_link = use_deep_link();
    rsx! {
        div {
            h1 { "Hello, deep links!" }
            p { "Last deep link: {deep_link:?}" }
        }
    }
}

fn main() {
    let nexum_config = Config::builder()
        .scheme("dioxus")
        .build()
        .expect("valid config");

    let desktop_config = DesktopConfig::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Deep Link Demo")
                .with_inner_size(LogicalSize::new(800.0, 600.0)),
        )
        .with_custom_head(
            r#"<link rel="preconnect" href="https://fonts.googleapis.com">"#.to_string(),
        );

    launch_desktop(&nexum_config, desktop_config, app);
}
