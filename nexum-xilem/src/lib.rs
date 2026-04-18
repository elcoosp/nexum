use nexum_core::{Config, DeepLinkHandle, DeepLinkHub};
use xilem::core::fork;

/// Initializes deep linking and returns a handle.
pub fn setup(config: Config) -> DeepLinkHandle {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();
    nexum_platform::set_hub(hub);
    nexum_platform::register(&config);
    handle
}

/// Wraps a view with a background deep link listener.
pub fn with_deep_links<State: 'static, V: xilem::WidgetView<State>>(
    view: V,
    handle: DeepLinkHandle,
    on_url: impl Fn(&mut State, String) + Send + Sync + 'static,
) -> impl xilem::WidgetView<State> {
    let task = xilem::view::task_raw(
        move |proxy| {
            let handle = handle.clone();
            async move {
                while let Some(url) = handle.recv().await {
                    if proxy.message(url).is_err() {
                        break;
                    }
                }
            }
        },
        on_url,
    );

    fork(view, task)
}
