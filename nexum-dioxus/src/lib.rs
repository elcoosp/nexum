use dioxus::prelude::*;
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// A global signal containing the most recent deep link URLs.
pub static DEEP_LINK_URLS: GlobalSignal<Option<Vec<Url>>> = Signal::global(|| None);

/// Initializes the deep link listener as a coroutine.
/// Call this from your root component.
pub fn use_deep_link_listener(config: Config) {
    let inner = CoreNexum::new(config);
    inner.register_all().expect("Failed to register schemes");

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let url_rx = inner.event_receiver();
        async move {
            while let Ok(urls) = url_rx.recv().await {
                *DEEP_LINK_URLS.write() = Some(urls);
            }
        }
    });
}

/// Alternative: returns a signal that updates with each deep link.
pub fn use_deep_link_signal(config: Config) -> Signal<Option<Vec<Url>>> {
    let mut signal = use_signal(|| None);
    let inner = CoreNexum::new(config);
    inner.register_all().expect("Failed to register schemes");

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let url_rx = inner.event_receiver();
        async move {
            while let Ok(urls) = url_rx.recv().await {
                signal.set(Some(urls));
            }
        }
    });

    signal
}

/// Returns the current deep link URLs, if any.
pub fn get_current(config: &Config) -> Option<Vec<Url>> {
    CoreNexum::new(config.clone()).get_current()
}

/// Checks if a scheme is registered.
pub fn is_registered(config: &Config, scheme: &str) -> Result<bool, nexum_core::Error> {
    CoreNexum::new(config.clone()).is_registered(scheme)
}

/// Unregisters a scheme (Windows/Linux only).
pub fn unregister(config: &Config, scheme: &str) -> Result<(), nexum_core::Error> {
    CoreNexum::new(config.clone()).unregister(scheme)
}

/// macOS/iOS: Call this from your native app delegate.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn handle_open_urls(urls: Vec<String>) {
    CoreNexum::handle_open_urls(urls);
}
