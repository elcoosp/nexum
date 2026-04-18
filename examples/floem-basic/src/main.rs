use floem::prelude::*;
use floem::reactive::create_rw_signal;
use floem::views::{label, v_stack};
use floem::View;
use nexum_core::Config;
use nexum_floem::setup;

fn app_view() -> impl View {
    let deep_link_urls = create_rw_signal(String::from("No deep link received"));
    let config = Config {
        schemes: vec!["floem".to_string()],
        app_links: vec![],
    };
    let handle = setup(config);

    let urls_signal = deep_link_urls;
    std::thread::spawn(move || {
        while let Ok(url) = handle.recv_blocking() {
            urls_signal.set(format!("Last URL: {}", url));
        }
    });

    v_stack((
        label(|| "Hello, deep links!".to_string()),
        label(move || deep_link_urls.get()),
    ))
}

fn main() {
    floem::launch(app_view);
}
