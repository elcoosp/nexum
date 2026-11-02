//! Xilem adapter for nexum deep linking.
//!
//! Call [`setup`] once at startup, then wrap any view with
//! [`with_deep_links`]; the callback runs on the app thread for every deep
//! link:
//!
//! ```no_run
//! use nexum_core::Config;
//! use nexum_xilem::{setup, with_deep_links};
//! use xilem::view::{flex_col, label};
//! use xilem::WidgetView;
//!
//! #[derive(Default)]
//! struct AppState {
//!     last_url: Option<String>,
//! }
//!
//! fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
//!     let content = flex_col((
//!         label("Hello, deep links!"),
//!         label(state.last_url.clone().unwrap_or_else(|| "No deep link received".to_owned())),
//!     ));
//!     with_deep_links(content, |state: &mut AppState, url| {
//!         state.last_url = Some(url);
//!     })
//! }
//!
//! fn main() -> Result<(), nexum_xilem::Error> {
//!     let config = Config::builder().scheme("xilem").build().expect("valid config");
//!     setup(&config)?;
//!     Ok(())
//! }
//! ```

use std::sync::OnceLock;

use xilem::WidgetView;
use xilem::core::fork;
use xilem::view::task_raw;

pub use nexum_core::{
    AppLink, Config, DeepLinkEvent, DeepLinkHandle, DeepLinkHub, Error, Result, Source,
};
pub use nexum_platform as platform;

static HANDLE: OnceLock<DeepLinkHandle> = OnceLock::new();

/// Initializes deep linking and stores the consumer handle for
/// [`with_deep_links`].
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
        .expect("nexum_xilem::setup must be called before consuming deep links")
}

/// Wraps a view with a background deep-link listener.
///
/// The `on_url` callback runs on the application thread whenever a deep link
/// arrives, exactly as an ordinary Xilem event callback would.
///
/// # Panics
///
/// Panics when [`setup`] has not been called yet.
pub fn with_deep_links<State: 'static, V: WidgetView<State>>(
    view: V,
    on_url: impl Fn(&mut State, String) + Send + Sync + 'static,
) -> impl WidgetView<State> {
    let handle = handle().clone();
    let task = task_raw(
        move |proxy| {
            let handle = handle.clone();
            async move { relay_urls(handle, proxy).await }
        },
        on_url,
    );

    fork(view, task)
}

/// Forwards every deep-link URL arriving on `handle` into the Xilem message
/// proxy until the channel shuts down or the view tree is gone.
///
/// Extracted from [`with_deep_links`] so it can be tested headlessly
/// against a mock [`RawProxy`](xilem::core::RawProxy).
async fn relay_urls(handle: DeepLinkHandle, proxy: xilem::core::MessageProxy<String>) {
    while let Some(url) = handle.recv().await {
        if proxy.message(url).is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    use futures::executor::block_on;
    use nexum_core::{Config, DeepLinkHub};
    use xilem::core::{MessageProxy, ProxyError, RawProxy, SendMessage, ViewId};
    use xilem::view::label;

    use super::{HANDLE, handle, relay_urls, setup, with_deep_links};

    /// A [`RawProxy`] that records the URLs it receives into a shared
    /// vector and can be told to report an expired driver.
    struct RecordingProxy {
        recorded: Arc<Mutex<Vec<String>>>,
        fail: Arc<AtomicBool>,
    }

    impl std::fmt::Debug for RecordingProxy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("RecordingProxy")
        }
    }

    impl RawProxy for RecordingProxy {
        fn send_message(
            &self,
            _path: Arc<[ViewId]>,
            message: SendMessage,
        ) -> Result<(), ProxyError> {
            if self.fail.load(Ordering::SeqCst) {
                return Err(ProxyError::DriverFinished(message));
            }
            let url = message
                .0
                .downcast::<String>()
                .map(|boxed| *boxed)
                .map_err(|original| ProxyError::DriverFinished(SendMessage(original)))?;
            self.recorded.lock().expect("poisoned lock").push(url);
            Ok(())
        }

        fn dyn_debug(&self) -> &dyn std::fmt::Debug {
            self
        }
    }

    fn proxy(recorded: &Arc<Mutex<Vec<String>>>, fail: &Arc<AtomicBool>) -> MessageProxy<String> {
        MessageProxy::new(
            Arc::new(RecordingProxy {
                recorded: Arc::clone(recorded),
                fail: Arc::clone(fail),
            }),
            Arc::from([]),
        )
    }

    /// The single test in this binary touching the `HANDLE` static.
    #[test]
    fn setup_handle_and_view_construction() {
        // Registration side effects are redirected into a temp dir.
        let _guard = isolate_registration();

        // Before setup the handle accessor is a programming error.
        assert!(HANDLE.get().is_none());

        let config = Config::builder()
            .scheme("nexumxilem")
            .build()
            .expect("valid config");
        setup(&config).expect("first setup");
        let error = setup(&config).expect_err("second setup must fail");
        assert!(matches!(error, nexum_core::Error::HubAlreadySet));

        // The handle accessor works after setup.
        assert_eq!(handle().try_recv(), None);

        // Building the wrapped view tree needs no window.
        let view = with_deep_links(label("ready"), |_state: &mut (), _url| {});
        let _ = view;
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

    #[test]
    fn relay_urls_forwards_until_channel_closes() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("nexumxilem://1");
        hub.push_url("nexumxilem://2");
        drop(hub);

        let recorded = Arc::new(Mutex::new(Vec::new()));
        let proxy = proxy(&recorded, &Arc::new(AtomicBool::new(false)));
        block_on(relay_urls(handle, proxy));

        assert_eq!(
            *recorded.lock().expect("poisoned lock"),
            vec!["nexumxilem://1".to_owned(), "nexumxilem://2".to_owned()]
        );
    }

    #[test]
    fn relay_urls_stops_when_the_view_expires() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("nexumxilem://1");
        hub.push_url("nexumxilem://2");
        drop(hub);

        let recorded = Arc::new(Mutex::new(Vec::new()));
        let fail = Arc::new(AtomicBool::new(true));
        let proxy = proxy(&recorded, &fail);
        block_on(relay_urls(handle, proxy));

        // The very first message reported an expired driver, so the relay
        // stopped without recording anything.
        assert!(recorded.lock().expect("poisoned lock").is_empty());
    }
}
