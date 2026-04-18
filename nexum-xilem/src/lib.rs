use nexum_core::{Config, DeepLinkHandle, DeepLinkHub};
use std::sync::OnceLock;
use xilem::core::fork;

static HANDLE: OnceLock<DeepLinkHandle> = OnceLock::new();

/// Initializes deep linking. Call this once at startup.
pub fn setup(config: Config) {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();
    HANDLE.set(handle).unwrap();
    nexum_platform::set_hub(hub);
    nexum_platform::register(&config);
}

/// Wraps a view with a background deep link listener.
/// The `on_url` callback is invoked whenever a new URL is received.
pub fn with_deep_links<State: 'static, V: xilem::WidgetView<State>>(
    view: V,
    on_url: impl Fn(&mut State, String) + Send + Sync + 'static,
) -> impl xilem::WidgetView<State> {
    let handle = HANDLE.get().expect("nexum_xilem::setup not called").clone();
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
