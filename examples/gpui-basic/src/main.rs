use gpui::*;
use nexum_core::Config;
use nexum_gpui::Nexum;

struct HelloWorld;

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(gpui::white())
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(gpui::black())
            .child("Hello, deep links!")
    }
}

fn main() {
    let config = Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };

    let nexum = Nexum::new(config);

    Application::new().run(move |cx: &mut App| {
        nexum.spawn_listener(cx);

        Nexum::on_deep_link(cx, |urls, _cx| {
            println!("Deep link received: {:?}", urls);
        });

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| HelloWorld)
        })
        .unwrap();
    });
}
