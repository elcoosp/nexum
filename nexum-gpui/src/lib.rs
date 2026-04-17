use async_channel::Receiver;
use gpui::{App, AsyncApp, BorrowAppContext, Global};
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

pub type DeepLinkCallback = Box<dyn Fn(&[Url], &mut App) + Send + Sync>;

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

pub struct Nexum {
    inner: CoreNexum,
}

impl Nexum {
    pub fn new(config: Config) -> Self {
        let inner = CoreNexum::new(config);

        #[cfg(not(target_os = "macos"))]
        inner.register_all().expect("Failed to register schemes");

        Self { inner }
    }

    pub fn event_receiver(&self) -> Receiver<Vec<Url>> {
        self.inner.event_receiver()
    }

    pub fn spawn_listener(&self, cx: &mut App) {
        if !cx.has_global::<DeepLinkRegistry>() {
            cx.set_global(DeepLinkRegistry {
                callbacks: Vec::new(),
            });
        }

        // Swizzle the delegate synchronously on the main thread.
        // By this point, application().run() has guaranteed the delegate exists.
        #[cfg(target_os = "macos")]
        nexum_core::platform::macos::setup_apple_event_listener();

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

    pub fn invoke_callbacks(urls: &[Url], cx: &mut App) {
        if cx.has_global::<DeepLinkRegistry>() {
            cx.update_global(|registry: &mut DeepLinkRegistry, cx| {
                registry.invoke(urls, cx);
            });
        }
    }

    pub fn get_current(&self) -> Option<Vec<Url>> {
        self.inner.get_current()
    }

    pub fn is_registered(&self, scheme: &str) -> Result<bool, nexum_core::Error> {
        self.inner.is_registered(scheme)
    }

    pub fn unregister(&self, scheme: &str) -> Result<(), nexum_core::Error> {
        self.inner.unregister(scheme)
    }
}
