use floem::{
    reactive::{create_effect, create_rw_signal, RwSignal},
    views::{label, v_stack, Decorators},
    View,
};
use nexum_core::Config;
use nexum_floem::spawn_deep_link_listener;
use url::Url;

fn app_view() -> impl View {
    let deep_link_urls: RwSignal<Option<Vec<Url>>> = create_rw_signal(None);
    let config = Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };
    spawn_deep_link_listener(config, deep_link_urls);

    let display_text = create_rw_signal("No deep link received".to_string());
    create_effect(move |_| {
        if let Some(urls) = deep_link_urls.get() {
            display_text.set(format!("Last URL: {:?}", urls));
        }
    });

    v_stack((
        label(|| "Hello, deep links!".to_string()),
        label(move || display_text.get()),
    ))
    .style(|s| s.padding(20.0))
}

fn main() {
    floem::launch(app_view);
}
