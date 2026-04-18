use async_channel::Sender;
use dioxus::prelude::*;
use std::sync::OnceLock;

// Re‑export core types and Dioxus desktop building blocks
pub use dioxus::LaunchBuilder;
pub use dioxus_desktop::Config as DesktopConfig;
pub use dioxus_desktop::{LogicalSize, WindowBuilder};
pub use nexum_core::{Config, DeepLinkHandle, DeepLinkHub}; // optional convenience

// Internal channel
static HANDLE: OnceLock<DeepLinkHandle> = OnceLock::new();
static HUB_SENDER: OnceLock<Sender<String>> = OnceLock::new();

/// Must be called once before using deep links.
/// It registers the URL schemes with the OS and sets up the global channel.
pub fn setup(config: Config) {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();
    HANDLE.set(handle).unwrap();
    HUB_SENDER.set(hub.sender()).unwrap();
    nexum_platform::set_hub(hub);
    nexum_platform::register(&config);
}

/// Send a URL into the global channel (used internally by the event handler).
pub fn push_deep_link(url: String) {
    println!("[nexum_dioxus] push_deep_link called with: {}", url);
    if let Some(sender) = HUB_SENDER.get() {
        let _ = sender.try_send(url);
    } else {
        println!("[nexum_dioxus] ERROR: HUB_SENDER not set");
    }
}

/// Hook to read the latest deep link inside a Dioxus component.
pub fn use_deep_link() -> Signal<Option<String>> {
    let mut signal = use_signal(|| None);
    let handle = HANDLE
        .get()
        .expect("nexum_dioxus::setup not called")
        .clone();

    use_future(move || {
        let handle = handle.clone();
        async move {
            while let Some(url) = handle.recv().await {
                signal.set(Some(url));
            }
        }
    });

    signal
}

/// Attach the deep‑link event handler to a `DesktopConfig`.
/// Use this if you want to build the config manually before launching.
pub fn with_deep_link_handler(mut config: DesktopConfig) -> DesktopConfig {
    config = config.with_custom_event_handler(|event, _| {
        if let tao::event::Event::Opened { urls } = event {
            if let Some(url) = urls.first() {
                push_deep_link(url.to_string());
            }
        }
    });
    config
}

/// One‑stop launch function: sets up deep links, attaches the handler, and launches the app.
/// The caller provides the full `DesktopConfig` (window, custom head, etc.).
pub fn launch_desktop(
    nexum_config: Config,
    desktop_config: DesktopConfig,
    app: fn() -> Element, // <-- function pointer, not generic
) {
    setup(nexum_config);
    let final_config = with_deep_link_handler(desktop_config);
    LaunchBuilder::new().with_cfg(final_config).launch(app);
}
