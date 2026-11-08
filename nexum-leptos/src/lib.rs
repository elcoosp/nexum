//! Leptos adapter for nexum deep linking.
//!
//! Two entry points are available:
//!
//! * [`setup`] returns a [`DeepLinkHandle`] for manual (blocking or async)
//!   consumption — always available.
//! * [`signal`] bridges deep links into a Leptos
//!   [`RwSignal<Option<String>>`] — available with the `signal` feature.
//!   [`signal_from_handle`] is the framework-agnostic core of the same
//!   bridge: it never touches the OS, so it also runs in the browser
//!   (`wasm32`), where scheme registration is the embedding platform's job.
//!
//! # Example (signal feature)
//!
//! ```no_run
//! use leptos::prelude::*;
//! use nexum_core::Config;
//! use nexum_leptos::signal;
//!
//! let config = Config::builder().scheme("myapp").build().expect("valid config");
//! // Updates on every deep-link event.
//! let url = signal(&config);
//!
//! // Inside a component, render the latest URL reactively:
//! // view! { <p>{move || url.get().unwrap_or_else(|| "waiting…".to_owned())}</p> }
//! # let _ = &url;
//! ```

// The reactive types are only reachable with the `signal` feature, which is
// also what pulls in the optional `leptos` dependency.
#[cfg(feature = "signal")]
use leptos::prelude::{RwSignal, Set};

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

/// Initializes deep linking and bridges events into a Leptos signal.
///
/// The signal starts as `None` and is replaced by every incoming URL. The
/// bridge consumes the handle through
/// [`spawn_local`](leptos::task::spawn_local), so it runs on the executor
/// Leptos initialized for the app (wasm-bindgen in the browser, tokio or
/// futures on native).
///
/// # Panics
///
/// Panics when platform registration fails; an application that cannot
/// register its schemes cannot receive deep links, so this is treated as a
/// fatal setup error. Use [`setup`] for a fallible API you can build your own
/// bridge on.
#[cfg(feature = "signal")]
#[must_use]
pub fn signal(config: &Config) -> RwSignal<Option<String>> {
    let handle = setup(config).expect("nexum-leptos: deep-link registration failed");
    signal_from_handle(&handle)
}

/// Bridges an existing [`DeepLinkHandle`] into a Leptos signal.
///
/// Unlike [`signal`] this never touches the OS, which makes it usable
/// everywhere Leptos compiles — including `wasm32` in the browser, where the
/// demo app in `examples/leptos-basic` feeds the hub directly. When an async
/// executor is required, it must already be initialized (Leptos does that in
/// `mount_to_body` on the web and through its SSR runtime on the server).
///
/// The returned signal starts as `None` and holds the latest URL afterwards.
#[cfg(feature = "signal")]
#[must_use]
pub fn signal_from_handle(handle: &DeepLinkHandle) -> RwSignal<Option<String>> {
    let signal = RwSignal::new(None);
    leptos::task::spawn_local(relay_to_signal(handle.clone(), signal));
    signal
}

/// Forwards every deep-link URL from `handle` into `signal` until the hub
/// shuts down (the handle's channel disconnects).
///
/// Extracted from [`signal_from_handle`] so the loop is testable without a
/// live Leptos runtime.
#[cfg(feature = "signal")]
pub async fn relay_to_signal(handle: DeepLinkHandle, signal: RwSignal<Option<String>>) {
    while let Some(url) = handle.recv().await {
        signal.set(Some(url));
    }
}

#[cfg(all(test, feature = "signal"))]
mod tests {
    use any_spawner::Executor;
    use futures::executor::block_on;
    use leptos::prelude::GetUntracked;

    use super::{
        Config, DeepLinkHub, Error, RwSignal, platform, relay_to_signal, setup, signal,
        signal_from_handle,
    };

    #[test]
    fn relay_to_signal_forwards_until_handle_shuts_down() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("myapp://first");
        hub.push_url("myapp://second");
        drop(hub);

        let url_signal = RwSignal::new(None);
        block_on(relay_to_signal(handle, url_signal));

        // The last URL before shutdown wins, then the loop ends gracefully.
        assert_eq!(
            url_signal.get_untracked().as_deref(),
            Some("myapp://second")
        );
    }

    /// The single test in this binary touching process-global state: the
    /// async executor, the platform hub and the mutated environment. Every
    /// global-dependent step therefore runs inside this one function, in a
    /// fixed order, on a tokio `LocalSet` (which is what `spawn_local`
    /// requires on native targets).
    #[cfg_attr(
        not(target_os = "linux"),
        ignore = "platform registration is exercised on Linux"
    )]
    #[test]
    fn signal_bridge_boots_registration_and_delivers_urls() {
        let _guard = isolate_registration();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("single-thread runtime");
        tokio::task::LocalSet::new().block_on(&runtime, async {
            // any_spawner must know how to spawn before the first bridge.
            Executor::init_tokio().expect("executor not yet initialized");

            // 1. The framework-agnostic bridge on a private hub.
            let hub = DeepLinkHub::new();
            let handle = hub.handle();
            let url_signal = signal_from_handle(&handle);
            assert_eq!(url_signal.get_untracked(), None);

            hub.push_url("myapp://runtime");
            for _ in 0..25 {
                tokio::task::yield_now().await;
            }
            assert_eq!(
                url_signal.get_untracked().as_deref(),
                Some("myapp://runtime")
            );

            // 2. The one-call `signal(&config)` boots the real platform
            //    registration and returns a live signal.
            let config = Config::builder()
                .scheme("nexumleptos")
                .build()
                .expect("valid config");
            let platform_signal = signal(&config);
            assert_eq!(platform_signal.get_untracked(), None);

            platform::push_url("nexumleptos://delivered").expect("hub set");
            for _ in 0..25 {
                tokio::task::yield_now().await;
            }
            assert_eq!(
                platform_signal.get_untracked().as_deref(),
                Some("nexumleptos://delivered")
            );

            // 3. A second registration cannot reinstall the global hub.
            let error = setup(&config).expect_err("already set");
            assert!(matches!(error, Error::HubAlreadySet));
        });
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
