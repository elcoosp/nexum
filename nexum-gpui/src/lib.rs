// nexum-gpui/src/lib.rs
use gpui::{App, AsyncApp, Entity};
use nexum_core::DeepLinkHandle;
pub use nexum_core::{Config, DeepLinkHub};
pub fn setup_deep_links(app: &gpui::Application, _config: Config) -> DeepLinkHandle {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();
    app.on_open_urls(move |urls| {
        for url in urls {
            hub.push_url(url);
        }
    });
    handle
}

pub fn attach_deep_link<V: 'static>(
    handle: DeepLinkHandle,
    view: Entity<V>,
    cx: &mut App,
    on_url: impl Fn(&mut V, String) + 'static,
) {
    cx.spawn({
        let view = view.clone();
        async move |cx: &mut AsyncApp| {
            while let Some(url) = handle.recv().await {
                let _ = cx.update(|cx| {
                    view.update(cx, |v, cx| {
                        on_url(v, url);
                        cx.notify();
                    });
                });
            }
        }
    })
    .detach();
}
