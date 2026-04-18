use gpui::*;
use gpui_platform::application;
use nexum_gpui::{attach_deep_link, setup_deep_links, Config};

struct DeepLinkApp {
    last_url: Option<String>,
}

impl DeepLinkApp {
    fn set_url(&mut self, url: String) {
        self.last_url = Some(url);
    }
}

impl Render for DeepLinkApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0xeeeeee))
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap(px(20.0))
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(0x000000))
                    .child("Hello, deep links!"),
            )
            .child(
                div()
                    .text_lg()
                    .text_color(rgb(0x333333))
                    .child(match &self.last_url {
                        Some(url) => format!("Received: {}", url),
                        None => "Waiting for deep link...".to_string(),
                    }),
            )
    }
}

fn main() {
    let app = application();
    let handle = setup_deep_links(
        &app,
        Config {
            schemes: vec!["myapp".to_string()],
            app_links: vec![], // Add this line
        },
    );

    app.run(move |cx: &mut App| {
        let view = cx.new(|_cx| DeepLinkApp { last_url: None });

        // One line – all the boilerplate is gone!
        attach_deep_link(handle.clone(), view.clone(), cx, |view, url| {
            view.set_url(url)
        });

        cx.open_window(WindowOptions::default(), |_, _cx| view)
            .unwrap();
        cx.activate(true);
    });
}
