use async_channel::Receiver;
use gpui::{App, AsyncApp, BorrowAppContext, Global};
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// A type alias for deep link callbacks.
pub type DeepLinkCallback = Box<dyn Fn(&[Url], &mut App) + Send + Sync>;

/// Global state holding registered callbacks.
pub struct DeepLinkRegistry {
    callbacks: Vec<DeepLinkCallback>,
}

impl Global for DeepLinkRegistry {}

impl DeepLinkRegistry {
    pub fn push(&mut self, cb: DeepLinkCallback) {
        self.callbacks.push(cb);
    }

    pub fn invoke(&self, urls: &[Url], cx: &mut App) {
        for cb in &self.callbacks {
            cb(urls, cx);
        }
    }
}

/// The Nexum GPUI adapter.
pub struct Nexum {
    inner: CoreNexum,
}

impl Nexum {
    /// Creates a new Nexum instance and registers all schemes.
    pub fn new(config: Config) -> Self {
        let inner = CoreNexum::new(config);
        inner.register_all().expect("Failed to register schemes");
        Self { inner }
    }

    /// Returns the event receiver for deep link URLs.
    pub fn event_receiver(&self) -> Receiver<Vec<Url>> {
        self.inner.event_receiver()
    }

    /// Spawns a background task that listens for URLs and invokes registered callbacks.
    pub fn spawn_listener(&self, cx: &mut App) {
        if !cx.has_global::<DeepLinkRegistry>() {
            cx.set_global(DeepLinkRegistry {
                callbacks: Vec::new(),
            });
        }

        let rx = self.inner.event_receiver();
        cx.spawn(async move |cx: &mut AsyncApp| {
            while let Ok(urls) = rx.recv().await {
                cx.update(|cx: &mut App| {
                    cx.update_global(|registry: &mut DeepLinkRegistry, cx| {
                        registry.invoke(&urls, cx);
                    });
                });
            }
        })
        .detach();
    }
    /// Registers a callback to be invoked when a deep link is received.
    pub fn on_deep_link(cx: &mut App, callback: impl Fn(&[Url], &mut App) + Send + Sync + 'static) {
        if !cx.has_global::<DeepLinkRegistry>() {
            cx.set_global(DeepLinkRegistry {
                callbacks: Vec::new(),
            });
        }
        cx.update_global(|registry: &mut DeepLinkRegistry, _| {
            registry.push(Box::new(callback));
        });
    }

    /// Manually invoke all registered callbacks with the given URLs.
    pub fn invoke_callbacks(urls: &[Url], cx: &mut App) {
        if cx.has_global::<DeepLinkRegistry>() {
            cx.update_global(|registry: &mut DeepLinkRegistry, cx| {
                registry.invoke(urls, cx);
            });
        }
    }

    /// Returns the current deep link URLs, if any.
    pub fn get_current(&self) -> Option<Vec<Url>> {
        self.inner.get_current()
    }

    /// Checks if a scheme is registered.
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
