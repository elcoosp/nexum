//! Nexum Dioxus adapter: deep linking integration for the Dioxus framework.
//!
//! Provides a [`GlobalSignal`] ([`DEEP_LINK_URLS`]) that is automatically
//! updated when deep links arrive, plus a [`use_deep_link_listener`] hook
//! that initialises the background listener as a Dioxus coroutine.
//!
//! # Example
//!
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use nexum_dioxus::{use_deep_link_listener, DEEP_LINK_URLS};
//! use nexum_core::Config;
//!
//! fn App() -> Element {
//!     let config = Config {
//!         schemes: vec!["myapp".to_string()],
//!         app_links: vec![],
//!     };
//!     use_deep_link_listener(config);
//!
//!     let urls = DEEP_LINK_URLS.read();
//!     rsx! {
//!         div {
//!             h1 { "Hello, deep links!" }
//!             p { match urls.as_ref() {
//!                 Some(u) => format!("{:?}", u),
//!                 None => "No deep link yet".into(),
//!             }}
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// A global signal containing the most recent deep link URLs.
///
/// Updated automatically when [`use_deep_link_listener`] is active.
pub static DEEP_LINK_URLS: GlobalSignal<Option<Vec<Url>>> = Signal::global(|| None);

/// Initialises the deep link listener as a Dioxus coroutine.
///
/// Call this from your root component. Incoming URLs will be written to
/// [`DEEP_LINK_URLS`].
pub fn use_deep_link_listener(config: Config) {
    let inner = CoreNexum::new(config);
    if let Err(e) = inner.register_all() {
        eprintln!("[nexum-dioxus] Failed to register schemes: {}", e);
    }

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let url_rx = inner.event_receiver();
        async move {
            while let Ok(urls) = url_rx.recv().await {
                *DEEP_LINK_URLS.write() = Some(urls);
            }
        }
    });
}

/// Returns a component-local signal that updates with each deep link.
///
/// Use this when you prefer a scoped signal over the global [`DEEP_LINK_URLS`].
pub fn use_deep_link_signal(config: Config) -> Signal<Option<Vec<Url>>> {
    let signal = use_signal(|| None);
    let inner = CoreNexum::new(config);
    if let Err(e) = inner.register_all() {
        eprintln!("[nexum-dioxus] Failed to register schemes: {}", e);
    }

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

/// Returns the current deep link URLs from CLI arguments, if any (Windows/Linux).
pub fn get_current(config: &Config) -> Option<Vec<Url>> {
    CoreNexum::new(config.clone()).get_current()
}

/// Checks if a scheme is registered as the default handler.
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
