//! Dioxus desktop adapter for nexum deep linking.
//!
//! * [`setup`] registers the schemes and stores the consumer handle.
//! * [`with_deep_link_handler`] wires tao's `Event::Opened` into the hub.
//! * [`launch_desktop`] is the one-call path: setup + handler + launch.
//! * [`use_deep_link`] exposes the latest URL as a
//!   [`Signal<Option<String>>`] inside components.
//!
//! # Example
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use dioxus_desktop::{Config as DesktopConfig, LogicalSize, WindowBuilder};
//! use nexum_dioxus::{launch_desktop, use_deep_link, Config};
//!
//! fn app() -> Element {
//!     let deep_link = use_deep_link();
//!     rsx! {
//!         div {
//!             h1 { "Hello, deep links!" }
//!             p { "Last deep link: {deep_link:?}" }
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let nexum_config = Config::builder().scheme("dioxus").build().expect("valid config");
//!     let desktop_config = DesktopConfig::new().with_window(
//!         WindowBuilder::new()
//!             .with_title("Deep Link Demo")
//!             .with_inner_size(LogicalSize::new(800.0, 600.0)),
//!     );
//!     launch_desktop(&nexum_config, desktop_config, app);
//! }
//! ```

use std::sync::OnceLock;

use dioxus::prelude::{Element, Signal, WritableExt, use_future, use_signal};
use tao::event::Event;

pub use dioxus::LaunchBuilder;
pub use dioxus_desktop::{Config as DesktopConfig, LogicalSize, WindowBuilder};
pub use nexum_core::{
    AppLink, Config, DeepLinkEvent, DeepLinkHandle, DeepLinkHub, Error, Result, Source,
};
pub use nexum_platform as platform;

static HANDLE: OnceLock<DeepLinkHandle> = OnceLock::new();

/// Initializes deep linking and stores the consumer handle for
/// [`use_deep_link`].
///
/// # Errors
///
/// Returns platform registration errors from
/// [`nexum_platform::init`](nexum_platform::init), or
/// [`Error::HubAlreadySet`] when this crate was already initialized.
pub fn setup(config: &Config) -> Result<()> {
    let handle = nexum_platform::init(config)?;
    HANDLE.set(handle).map_err(|_| Error::HubAlreadySet)
}

/// Returns the handle installed by [`setup`].
///
/// # Panics
///
/// Panics when [`setup`] has not been called yet; that is a programming
/// error, not a runtime condition.
#[must_use]
pub fn handle() -> &'static DeepLinkHandle {
    HANDLE
        .get()
        .expect("nexum_dioxus::setup must be called before consuming deep links")
}

/// Hook exposing the latest deep-link URL inside a Dioxus component.
///
/// The signal starts as `None` and is replaced by every incoming URL.
///
/// # Panics
///
/// Panics when [`setup`] has not been called yet.
#[must_use]
pub fn use_deep_link() -> Signal<Option<String>> {
    let mut signal = use_signal(|| None);
    let handle = handle().clone();

    use_future(move || {
        let handle = handle.clone();
        async move {
            relay_to_callback(handle, |url| signal.set(Some(url))).await;
        }
    });

    signal
}

/// Relays every deep-link URL from `handle` into `callback` until the
/// channel shuts down.
///
/// Extracted from [`use_deep_link`] so the loop is testable without a
/// Dioxus runtime.
async fn relay_to_callback(handle: DeepLinkHandle, mut callback: impl FnMut(String)) {
    while let Some(url) = handle.recv().await {
        callback(url);
    }
}

/// Attaches the deep-link event handler to a [`DesktopConfig`].
///
/// Use this when you build the config manually before launching; every URL
/// the window manager hands over is pushed into the hub.
#[must_use]
pub fn with_deep_link_handler(config: DesktopConfig) -> DesktopConfig {
    config.with_custom_event_handler(|event, _| {
        if let Event::Opened { urls } = event {
            for url in urls {
                nexum_platform::push_url(url.to_string()).ok();
            }
        }
    })
}

/// One-stop launch function: sets up deep links, attaches the tao event
/// handler and launches the desktop application.
///
/// # Panics
///
/// Panics when platform registration fails; a desktop app cannot start
/// without its window anyway, so this is treated as a fatal setup error.
pub fn launch_desktop(config: &Config, desktop_config: DesktopConfig, app: fn() -> Element) {
    setup(config).expect("nexum-dioxus: deep-link setup failed");
    let final_config = with_deep_link_handler(desktop_config);
    LaunchBuilder::new().with_cfg(final_config).launch(app);
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use futures::executor::block_on;
    use nexum_core::{Config, DeepLinkHub};

    use super::{DesktopConfig, HANDLE, handle, relay_to_callback, setup, with_deep_link_handler};

    /// The single test in this binary touching the `HANDLE` static.
    #[test]
    fn setup_and_handler_wiring() {
        let _guard = isolate_registration();

        assert!(HANDLE.get().is_none());
        let config = Config::builder()
            .scheme("nexumdioxus")
            .build()
            .expect("valid config");
        setup(&config).expect("first setup");
        let error = setup(&config).expect_err("second setup must fail");
        assert!(matches!(error, nexum_core::Error::HubAlreadySet));

        // The stored handle receives runtime URLs pushed via the platform.
        nexum_platform::push_url("nexumdioxus://runtime").expect("hub set");
        assert_eq!(
            handle().try_recv().as_deref(),
            Some("nexumdioxus://runtime")
        );

        // The event handler wrapper accepts and returns a desktop config.
        let desktop_config = DesktopConfig::new();
        let _wired = with_deep_link_handler(desktop_config);
    }

    #[test]
    fn relay_to_callback_forwards_until_channel_closes() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("nexumdioxus://1");
        hub.push_url("nexumdioxus://2");
        drop(hub);

        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        block_on(relay_to_callback(handle, move |url| {
            sink.lock().expect("poisoned lock").push(url);
        }));

        assert_eq!(
            *seen.lock().expect("poisoned lock"),
            vec!["nexumdioxus://1".to_owned(), "nexumdioxus://2".to_owned()]
        );
    }

    /// Redirects registration side effects into a temp dir: `XDG_DATA_HOME`
    /// points into it and a fake `xdg-mime` that succeeds instantly is
    /// prepended to `PATH`. The returned temp dir must be kept alive.
    fn isolate_registration() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bin = tmp.path().join("bin");
        std::fs::create_dir_all(&bin).expect("create bin dir");
        std::fs::write(bin.join("xdg-mime"), "#!/bin/sh\nexit 0\n").expect("write script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(bin.join("xdg-mime"), std::fs::Permissions::from_mode(0o755))
                .expect("chmod xdg-mime");
        }
        // SAFETY: the tests in this binary that care about the environment
        // run sequentially inside this single test function; no other test
        // thread reads the environment.
        unsafe {
            std::env::set_var("XDG_DATA_HOME", tmp.path().join("data"));
            let path = std::env::var("PATH").unwrap_or_default();
            std::env::set_var("PATH", format!("{}:{path}", bin.display()));
        }
        tmp
    }
}
