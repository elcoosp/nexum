//! GPUI adapter for nexum deep linking.
//!
//! GPUI surfaces incoming URLs through its own `on_open_urls` application
//! callback; this adapter bridges that callback into a nexum
//! [`DeepLinkHandle`] and additionally performs the OS-level scheme
//! registration needed on Windows and Linux:
//!
//! ```no_run
//! use gpui::Application;
//! use nexum_core::Config;
//! use nexum_gpui::setup_deep_links;
//!
//! let app = Application::headless();
//! let config = Config::builder().scheme("myapp").build().expect("valid config");
//! let handle = setup_deep_links(&app, &config).expect("deep links failed");
//!
//! app.run(move |cx| {
//!     let view = cx.new(|_| ());
//!     nexum_gpui::attach_deep_link(
//!         handle,
//!         view,
//!         cx,
//!         |_view: &mut (), _cx: &mut gpui::Context<()>, url| {
//!             println!("opened {url}");
//!         },
//!     );
//! });
//! ```

use gpui::{App, AsyncApp, Entity};
use nexum_core::{Config, DeepLinkHandle, Result};

/// Initializes deep linking for a GPUI application.
///
/// Installs the process-global nexum hub, performs the OS-level scheme
/// registration (registry on Windows, desktop entry on Linux, Apple events
/// on macOS) and forwards GPUI's `on_open_urls` callback into the hub.
///
/// # Errors
///
/// Returns platform registration errors from
/// [`nexum_platform::init`](nexum_platform::init), including
/// [`nexum_core::Error::HubAlreadySet`] when called twice.
pub fn setup_deep_links(app: &gpui::Application, config: &Config) -> Result<DeepLinkHandle> {
    let hub = nexum_core::DeepLinkHub::new();
    let handle = hub.handle();
    nexum_platform::set_hub(hub)?;

    // GPUI's `on_open_urls` callback takes the URLs only.
    app.on_open_urls(move |urls| {
        for url in urls {
            nexum_platform::push_url(url.to_string()).ok();
        }
    });

    nexum_platform::register(config)?;
    Ok(handle)
}

/// Drives `on_url` on the given entity for every deep link received on
/// `handle`.
///
/// The task lives for as long as the application runs (or until the handle's
/// channel shuts down).
pub fn attach_deep_link<V: 'static>(
    handle: DeepLinkHandle,
    view: Entity<V>,
    cx: &mut App,
    on_url: impl Fn(&mut V, &mut gpui::Context<V>, String) + 'static,
) {
    cx.spawn({
        async move |cx: &mut AsyncApp| {
            while let Some(url) = handle.recv().await {
                let _ = cx.update(|cx: &mut App| {
                    view.update(cx, |v, cx| {
                        on_url(v, cx, url);
                        cx.notify();
                    });
                });
            }
        }
    })
    .detach();
}
