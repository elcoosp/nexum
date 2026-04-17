use gpui::*;
use gpui_platform::application;
use nexum_core::Config;
use nexum_gpui::Nexum;

struct HelloWorld {
    deep_link: Option<String>,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(gpui::white())
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(gpui::black())
            .child("Hello, deep links!")
            .child(
                div()
                    .mt_4()
                    .p_4()
                    .rounded_md()
                    .bg(gpui::black().opacity(0.05))
                    .text_sm()
                    .text_color(rgb(0x666666))
                    .child(match &self.deep_link {
                        Some(url) => format!("Received: {}", url),
                        None => "Waiting for deep link...".to_string(),
                    }),
            )
    }
}
fn main() {
    let config = Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };

    let nexum = Nexum::new(config);

    // No tokio runtime or .enter() before this!
    application().run(move |cx: &mut App| {
        let view_entity = cx.new(|_| HelloWorld { deep_link: None });

        Nexum::on_deep_link(cx, {
            let view_entity = view_entity.clone();
            move |urls, cx| {
                if let Some(url) = urls.first() {
                    view_entity.update(cx, |view, cx| {
                        view.deep_link = Some(url.to_string());
                        cx.notify();
                    });
                }
            }
        });

        nexum.spawn_listener(cx);

        cx.open_window(WindowOptions::default(), |_, _cx| view_entity.clone())
            .unwrap();

        cx.activate(true);
    });
}
