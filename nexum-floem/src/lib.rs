//! Nexum Floem adapter: deep linking integration for the Floem UI framework.
//!
//! Uses Floem's [`RwSignal`] to deliver incoming deep-link URLs reactively.
//! A background thread blocks on the async channel and updates the signal,
//! which can be observed in a view's `update` closure or via [`create_effect`].
//!
//! # Example
//!
//! ```rust,no_run
//! use floem::reactive::{create_rw_signal, create_effect, RwSignal};
//! use nexum_floem::spawn_deep_link_listener;
//! use nexum_core::Config;
//! use url::Url;
//!
//! fn app_view() -> impl floem::View {
//!     let deep_link_urls: RwSignal<Option<Vec<Url>>> = create_rw_signal(None);
//!     let config = Config {
//!         schemes: vec!["myapp".to_string()],
//!         app_links: vec![],
//!     };
//!     spawn_deep_link_listener(config, deep_link_urls);
//!
//!     let display = create_rw_signal("No deep link".to_string());
//!     create_effect(move |_| {
//!         if let Some(urls) = deep_link_urls.get() {
//!             display.set(format!("URLs: {:?}", urls));
//!         }
//!     });
//!
//!     floem::views::label(move || display.get())
//! }
//! ```

use floem::reactive::RwSignal;
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// Spawns a background thread that listens for deep link URLs and updates the
/// given [`RwSignal`] whenever a new batch arrives.
///
/// The signal can be observed in a Floem view's `update` method or inside a
/// [`create_effect`](floem::reactive::create_effect).
pub fn spawn_deep_link_listener(config: Config, signal: RwSignal<Option<Vec<Url>>>) {
    let inner = CoreNexum::new(config);
    if let Err(e) = inner.register_all() {
        eprintln!("[nexum-floem] Failed to register schemes: {}", e);
    }

    std::thread::spawn(move || {
        let rx = inner.event_receiver();
        while let Ok(urls) = rx.recv_blocking() {
            signal.set(Some(urls));
        }
    });
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
