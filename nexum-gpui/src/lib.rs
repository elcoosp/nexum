use gpui::{App, AsyncApp, Entity};
use nexum_core::{Config, DeepLinkHandle, DeepLinkHub};

pub fn setup_deep_links(app: &gpui::Application, _config: Config) -> DeepLinkHandle {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();

    // GPUI's on_open_urls callback takes exactly 1 argument (the urls)
    app.on_open_urls(move |urls| {
        for url in urls {
            hub.push_url(url.to_string());
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
