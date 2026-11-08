//! Floem adapter for nexum deep linking.
//!
//! Two entry points are available:
//!
//! * [`setup`] returns a [`DeepLinkHandle`] for manual (blocking or async)
//!   consumption — always available.
//! * [`signal`] bridges deep links into a Floem
//!   `ReadSignal<Option<String>>` — available with the `signal` feature.
//!
//! # Example (signal feature)
//!
//! ```no_run
//! use floem::reactive::{ReadSignal, SignalGet};
//! use nexum_core::Config;
//! use nexum_floem::signal;
//!
//! let config = Config::builder().scheme("myapp").build().expect("valid config");
//! // Updates on every deep-link event.
//! let url_signal: ReadSignal<Option<String>> = signal(&config);
//!
//! floem::launch(move || {
//!     let url_signal = url_signal;
//!     floem::views::v_stack((floem::views::label(move || {
//!         url_signal.get().unwrap_or_else(|| "No deep link received".to_owned())
//!     }),))
//! });
//! ```

pub use nexum_core::{
    AppLink, Config, DeepLinkEvent, DeepLinkHandle, DeepLinkHub, Error, Result, Source,
};
pub use nexum_platform as platform;

/// Initializes deep linking and returns a consumer handle.
///
/// This is a thin wrapper over [`nexum_platform::init`]; use it when your
/// application wants to drive consumption itself (blocking, async or via its
/// own bridge).
///
/// # Errors
///
/// Returns the platform registration errors of
/// [`nexum_platform::init`], including
/// [`Error::HubAlreadySet`] when deep links were initialized twice.
pub fn setup(config: &Config) -> Result<DeepLinkHandle> {
    nexum_platform::init(config)
}

/// Initializes deep linking and bridges events into a Floem signal.
///
/// A background thread (named `nexum-floem`) forwards each URL into a
/// latest-value mailbox, and Floem's
/// [`create_signal_from_channel`](floem::ext_event::create_signal_from_channel)
/// wakes the UI thread only when a new URL arrives.
///
/// # Panics
///
/// Panics when platform registration fails or the bridge thread cannot be
/// spawned; both are unrecoverable startup errors for a UI application. Use
/// [`setup`] for a fallible API you can build your own bridge on.
#[cfg(feature = "signal")]
#[must_use]
pub fn signal(config: &Config) -> floem::reactive::ReadSignal<Option<String>> {
    use crossbeam_channel::bounded;
    use floem::ext_event::create_signal_from_channel;

    let handle = setup(config).expect("nexum-floem: deep-link registration failed");
    let (tx, rx) = bounded(1);

    std::thread::Builder::new()
        .name("nexum-floem".to_owned())
        .spawn(move || relay_urls(&handle, &tx))
        .expect("nexum-floem: failed to spawn the bridge thread");

    create_signal_from_channel(rx)
}

/// Forwards every deep-link URL from `handle` into `tx`.
///
/// Stops when the hub shuts down (the handle's channel disconnects) or when
/// the receiving end is dropped — the bridge thread then simply ends.
#[cfg(feature = "signal")]
fn relay_urls(handle: &DeepLinkHandle, tx: &crossbeam_channel::Sender<String>) {
    while let Ok(url) = handle.recv_blocking() {
        if tx.send(url).is_err() {
            break;
        }
    }
}

#[cfg(all(test, feature = "signal"))]
mod tests {
    use std::time::Duration;

    use crossbeam_channel::bounded;
    use floem::reactive::SignalGet;

    use super::*;

    #[test]
    fn relay_urls_forwards_until_handle_shuts_down() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("myapp://first");

        let (tx, rx) = bounded(1);
        let bridge = std::thread::spawn(move || relay_urls(&handle, &tx));

        let url = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("url forwarded");
        assert_eq!(url, "myapp://first");

        // Shutting the hub down ends the bridge thread gracefully.
        drop(hub);
        bridge.join().expect("bridge thread ended");
    }

    #[test]
    fn relay_urls_stops_when_the_receiver_drops() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();

        let (tx, rx) = bounded(1);
        let bridge = std::thread::spawn(move || relay_urls(&handle, &tx));

        drop(rx);
        hub.push_url("myapp://ignored");
        hub.push_url("myapp://also-ignored");

        // The bridge notices the closed channel and ends instead of
        // blocking forever.
        bridge
            .join()
            .expect("bridge ends after the receiver is dropped");
    }

    /// The single test touching the process-global hub: `signal` boots it
    /// through `setup` → `nexum_platform::init`.
    #[cfg_attr(
        not(target_os = "linux"),
        ignore = "platform registration is exercised on Linux"
    )]
    #[test]
    fn signal_boots_registration_and_creates_a_signal() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let fake_bin = tmp.path().join("bin");
        std::fs::create_dir_all(&fake_bin).expect("bin dir");

        // Fake `xdg-mime` that succeeds without touching the system.
        let fake_tool = fake_bin.join("xdg-mime");
        std::fs::write(&fake_tool, "#!/bin/sh\nexit 0\n").expect("script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake_tool, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        // SAFETY: process-global env; this test binary only mutates it from
        // this single test before any platform call.
        unsafe {
            std::env::set_var("XDG_DATA_HOME", tmp.path().join("data"));
            let path = std::env::var("PATH").unwrap_or_default();
            std::env::set_var("PATH", format!("{}:{path}", fake_bin.display()));
        }

        let config = Config::builder()
            .scheme("nexumfloem")
            .build()
            .expect("valid");

        // First boot creates the signal; reading it needs no event loop.
        let url_signal = signal(&config);
        assert!(url_signal.get().is_none());

        // Pushing a URL wakes the bridge thread (the ReadSignal itself only
        // updates inside a live Floem event loop).
        platform::push_url("nexumfloem://runtime").expect("hub set");

        // A second boot cannot reinstall the global hub.
        let error = setup(&config).expect_err("already set");
        assert!(matches!(error, Error::HubAlreadySet));

        // The signal stays readable and empty without an event loop.
        assert!(url_signal.get().is_none());
    }
}
