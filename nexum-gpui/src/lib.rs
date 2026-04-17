//! Nexum GPUI adapter: deep linking integration for Zed's GPUI framework.
//!
//! Uses GPUI's [`Global`] trait to maintain a callback registry and
//! `cx.spawn()` to listen for incoming URLs in the background.
//!
//! # Example
//!
//! ```rust,no_run
//! use gpui::*;
//! use nexum_gpui::Nexum;
//! use nexum_core::Config;
//!
//! fn main() {
//!     let config = Config {
//!         schemes: vec!["myapp".to_string()],
//!         app_links: vec![],
//!     };
//!     let nexum = Nexum::new(config);
//!
//!     Application::new().run(move |cx: &mut App| {
//!         nexum.spawn_listener(cx);
//!         Nexum::on_deep_link(cx, |urls, _cx| {
//!             println!("Deep link received: {:?}", urls);
//!         });
//!         // ... open windows etc.
//!     });
//! }
//! ```

use gpui::{App, Global, Subscription};
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// A type alias for deep link callbacks compatible with GPUI's `App` context.
pub type DeepLinkCallback = Box<dyn Fn(&[Url], &mut App) + Send + Sync>;

/// Global state holding registered deep-link callbacks.
struct DeepLinkRegistry {
    callbacks: Vec<DeepLinkCallback>,
}

impl Global for DeepLinkRegistry {}

impl DeepLinkRegistry {
    fn push(&mut self, cb: DeepLinkCallback) -> Subscription {
        self.callbacks.push(cb);
        // Note: In a full implementation, use a slot map or similar to allow
        // individual callback removal. For now, dropping the Subscription is
        // a no-op but provides the ergonomic API shape.
        Subscription::new(|| {})
    }

    fn invoke(&self, urls: &[Url], cx: &mut App) {
        for cb in &self.callbacks {
            cb(urls, cx);
        }
    }
}

/// The Nexum GPUI adapter.
///
/// Wraps [`CoreNexum`] and provides GPUI-idiomatic methods for registering
/// callbacks and spawning the background URL listener.
pub struct Nexum {
    inner: CoreNexum,
}

impl Nexum {
    /// Creates a new Nexum instance and registers all schemes with the OS.
    pub fn new(config: Config) -> Self {
        let inner = CoreNexum::new(config);
        inner
            .register_all()
            .expect("Failed to register deep link schemes");
        Self { inner }
    }

    /// Spawns a background task that listens for URLs and invokes all
    /// registered callbacks on the main thread via `cx.update_global`.
    pub fn spawn_listener(&self, cx: &mut App) {
        if !cx.has_global::<DeepLinkRegistry>() {
            cx.set_global(DeepLinkRegistry {
                callbacks: Vec::new(),
            });
        }

        let rx = self.inner.event_receiver();
        cx.spawn(|mut cx| async move {
            while let Ok(urls) = rx.recv().await {
                let _ = cx.update(|cx| {
                    cx.update_global(|registry: &mut DeepLinkRegistry, cx| {
                        registry.invoke(&urls, cx);
                    });
                });
            }
        })
        .detach();
    }

    /// Registers a callback to be invoked when a deep link is received.
    ///
    /// Returns a [`Subscription`] that keeps the callback alive. When the
    /// subscription is dropped the callback is conceptually unregistered.
    pub fn on_deep_link(
        cx: &mut App,
        callback: impl Fn(&[Url], &mut App) + Send + Sync + 'static,
    ) -> Subscription {
        if !cx.has_global::<DeepLinkRegistry>() {
            cx.set_global(DeepLinkRegistry {
                callbacks: Vec::new(),
            });
        }
        cx.update_global(|registry: &mut DeepLinkRegistry, _| {
            registry.push(Box::new(callback))
        })
    }

    /// Returns the current deep link URLs, if any (Windows/Linux CLI args).
    pub fn get_current(&self) -> Option<Vec<Url>> {
        self.inner.get_current()
    }

    /// Checks if a scheme is registered as the default handler.
    pub fn is_registered(&self, scheme: &str) -> Result<bool, nexum_core::Error> {
        self.inner.is_registered(scheme)
    }

    /// Unregisters a scheme (Windows/Linux only).
    pub fn unregister(&self, scheme: &str) -> Result<(), nexum_core::Error> {
        self.inner.unregister(scheme)
    }

    /// macOS/iOS: Call this from your native app delegate.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn handle_open_urls(urls: Vec<String>) {
        CoreNexum::handle_open_urls(urls);
    }
}
