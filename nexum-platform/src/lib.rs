//! OS-level URL-scheme registration for nexum.
//!
//! This crate owns the process-global [`DeepLinkHub`] and performs the
//! platform-specific registration of URL schemes:
//!
//! | Platform  | Mechanism                                   | Cold-start URL     |
//! |-----------|---------------------------------------------|--------------------|
//! | Windows   | Per-user registry (`Software\Classes`)      | command-line args  |
//! | macOS     | Apple events (`GURL`/`GURL`)                | Apple event        |
//! | Linux     | `.desktop` entry + `xdg-mime`               | command-line args  |
//!
//! Applications normally call one helper — [`init`] — which creates the
//! global hub, registers the schemes and returns the consumer handle:
//!
//! ```no_run
//! use nexum_core::Config;
//! use nexum_platform::init;
//!
//! let config = Config::builder().scheme("myapp").build().expect("valid config");
//! let handle = init(&config).expect("registration failed");
//!
//! // Blocking or async access to incoming deep links:
//! if let Ok(url) = handle.recv_blocking() {
//!     println!("opened {url}");
//! }
//! ```

use std::sync::OnceLock;

use nexum_core::{Config, DeepLinkHandle, DeepLinkHub, Error, Result};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

static HUB: OnceLock<DeepLinkHub> = OnceLock::new();

/// Returns the global hub if it has been initialized.
#[must_use]
pub fn hub() -> Option<&'static DeepLinkHub> {
    HUB.get()
}

/// Installs the process-global [`DeepLinkHub`].
///
/// # Errors
///
/// Returns [`Error::HubAlreadySet`] when a hub has already been installed
/// for this process.
pub fn set_hub(hub: DeepLinkHub) -> Result<()> {
    HUB.set(hub).map_err(|_| Error::HubAlreadySet)
}

/// Pushes a runtime deep-link URL into the global hub.
///
/// Use [`push_launch_url`] for URLs discovered on the command line so
/// applications can distinguish cold-start from runtime events.
///
/// # Errors
///
/// Returns [`Error::HubNotSet`] when [`init`] (or [`set_hub`]) has not run
/// yet.
pub fn push_url(url: impl Into<String>) -> Result<()> {
    let hub = HUB.get().ok_or(Error::HubNotSet)?;
    hub.push_url(url);
    Ok(())
}

/// Pushes a cold-start (command-line) deep-link URL into the global hub.
///
/// # Errors
///
/// Returns [`Error::HubNotSet`] when [`init`] (or [`set_hub`]) has not run
/// yet.
pub fn push_launch_url(url: impl Into<String>) -> Result<()> {
    let hub = HUB.get().ok_or(Error::HubNotSet)?;
    hub.push_launch_url(url);
    Ok(())
}

/// Performs all platform-specific registration for the given config.
///
/// * **Windows** — creates `Software\Classes\<scheme>` registry keys
///   (protocol marker, `DefaultIcon`, `shell\open\command`).
/// * **Linux** — writes `<app>.desktop` into `XDG_DATA_HOME/applications`
///   and wires every scheme with `xdg-mime`.
/// * **macOS** — installs the Apple-event handler for `GURL` events
///   (`Info.plist` scheme declarations must be provided by the bundle).
///
/// On Windows and Linux the command line is scanned for cold-start URLs and
/// each match is pushed with [`Source::CommandLine`](nexum_core::Source).
///
/// # Errors
///
/// Returns any error reported by the platform backend (I/O, missing tools,
/// registry failures).
///
/// # Panics
///
/// On macOS this panics when not called from the main thread.
pub fn register(config: &Config) -> Result<()> {
    #[cfg(target_os = "windows")]
    windows::register(config)?;
    #[cfg(target_os = "linux")]
    linux::register(config)?;
    #[cfg(target_os = "macos")]
    macos::register(config)?;

    // Cold-start URLs arrive as command-line arguments on Windows and Linux.
    #[cfg(not(target_os = "macos"))]
    relay_cold_start_urls(config, std::env::args().skip(1))?;
    Ok(())
}

/// Scans command-line arguments for URLs matching the configured schemes
/// and pushes each one into the global hub as a cold-start event.
fn relay_cold_start_urls<I>(config: &Config, args: I) -> Result<()>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    for url in config.extract_launch_urls(args) {
        push_launch_url(url)?;
    }
    Ok(())
}

/// One-call bootstrap: creates the global hub, registers the URL schemes
/// with the operating system and returns the consumer handle.
///
/// This is the single entry point every nexum adapter uses; applications
/// using `nexum-core` directly should call it too.
///
/// # Errors
///
/// Returns [`Error::HubAlreadySet`] when the hub was already installed, or
/// any platform registration error (see [`register`]).
pub fn init(config: &Config) -> Result<DeepLinkHandle> {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();
    set_hub(hub)?;
    register(config)?;
    Ok(handle)
}

#[cfg(test)]
mod integration {
    use std::path::Path;

    use super::*;
    use nexum_core::Source;

    /// The single test in this binary that touches the global hub.
    ///
    /// Everything depending on the `OnceLock` global must stay inside this
    /// test, in this exact order, so parallel tests never fight over
    /// process-wide state.
    #[test]
    fn full_init_flow_registers_and_delivers() {
        // Before any init the global hub is absent and pushes fail loudly.
        assert!(matches!(push_url("myapp://x"), Err(Error::HubNotSet)));
        assert!(matches!(
            push_launch_url("myapp://x"),
            Err(Error::HubNotSet)
        ));
        assert!(hub().is_none());

        let tmp = tempfile::tempdir().expect("tempdir");
        let fake_bin = tmp.path().join("bin");
        std::fs::create_dir_all(&fake_bin).expect("bin dir");
        let log_path = tmp.path().join("xdg-mime.log");

        // Fake `xdg-mime` that records its arguments and exits with the
        // status from `NEXUM_FAKE_XDG_MIME_RC` (default 0); prepended to PATH
        // so `register` deterministically calls it instead of the real tool.
        let fake_script = format!(
            "#!/bin/sh\necho \"$@\" >> {}\nexit ${{NEXUM_FAKE_XDG_MIME_RC:-0}}\n",
            log_path.display()
        );
        let fake_tool = fake_bin.join("xdg-mime");
        std::fs::write(&fake_tool, fake_script).expect("script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake_tool, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        // SAFETY: tests run in one thread per binary here; the env is only
        // mutated before any platform call happens in this test process.
        unsafe {
            std::env::set_var("XDG_DATA_HOME", tmp.path().join("data"));
            let path = std::env::var("PATH").unwrap_or_default();
            std::env::set_var("PATH", format!("{}:{path}", fake_bin.display()));
        }

        let config = Config::builder()
            .scheme("nexumtest")
            .icon_path("/icons/nexum.png")
            .build()
            .expect("valid config");

        // First init succeeds and returns a usable handle.
        let handle = init(&config).expect("init ok");

        // Runtime push + cold-start push are both delivered.
        push_url("nexumtest://runtime").expect("hub set");
        push_launch_url("nexumtest://cold").expect("hub set");
        let first = handle.recv_event_blocking().expect("event");
        let second = handle.recv_event_blocking().expect("event");
        assert_eq!(first.url, "nexumtest://runtime");
        assert_eq!(first.source, Source::System);
        assert_eq!(second.url, "nexumtest://cold");
        assert_eq!(second.source, Source::CommandLine);

        // Desktop entry was written into the redirected data dir.
        let exe = std::env::current_exe().expect("current exe");
        let app_name = exe
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .expect("file stem");
        let entry_path = tmp
            .path()
            .join("data/applications")
            .join(format!("{app_name}.desktop"));
        let entry = std::fs::read_to_string(&entry_path).expect("desktop entry");
        assert!(entry.contains("[Desktop Entry]"));
        assert!(entry.contains(&format!("Name={app_name}")));
        assert!(entry.contains("Icon=/icons/nexum.png"));
        assert!(entry.contains("x-scheme-handler/nexumtest"));

        // `xdg-mime default <file> <mime>` ran once per scheme.
        let log = std::fs::read_to_string(&log_path).expect("xdg-mime log");
        let expected = format!("default {app_name}.desktop x-scheme-handler/nexumtest\n");
        assert_eq!(log, expected);

        // A second hub cannot be installed.
        let error = init(&config).expect_err("already set");
        assert!(matches!(error, Error::HubAlreadySet));

        // Global accessor sees the hub.
        assert!(hub().is_some());

        // Cold-start relay: command-line URLs are pushed with the
        // command-line source.
        relay_cold_start_urls(&config, ["nexumtest://cli", "--flag"]).expect("relay ok");
        let third = handle.recv_event_blocking().expect("event");
        assert_eq!(third.url, "nexumtest://cli");
        assert_eq!(third.source, Source::CommandLine);

        // Linux-only: error paths of the desktop-entry registration.
        #[cfg(target_os = "linux")]
        linux_error_paths(&config, &tmp);
    }

    /// Covers the failure branches of the Linux registration backend with
    /// explicit inputs (no global state beyond the fake tool on `PATH`, so
    /// it stays inside the single serial test).
    #[cfg(target_os = "linux")]
    fn linux_error_paths(config: &Config, tmp: &tempfile::TempDir) {
        // No XDG data directory available.
        let error = linux::write_registration(None, Path::new("/opt/myapp"), config)
            .expect_err("no data dir");
        assert!(matches!(error, Error::Platform(_)));
        assert!(error.to_string().contains("XDG data directory"));

        // An executable name that sanitizes to nothing falls back to
        // `nexum-app` and the desktop file is still written.
        let data = tmp.path().join("fallback-data");
        linux::write_registration(
            Some(data.clone()),
            Path::new("###"),
            &Config::new(["nexumtest"]),
        )
        .expect("fallback registration");
        let entry_path = data.join("applications/nexum-app.desktop");
        let entry = std::fs::read_to_string(&entry_path).expect("fallback entry");
        assert!(entry.contains("Name=nexum-app\n"));

        // A failing `xdg-mime` surfaces as a platform error.
        // SAFETY: this only ever runs inside the single serial test.
        unsafe {
            std::env::set_var("NEXUM_FAKE_XDG_MIME_RC", "3");
        }
        let error = linux::write_registration(
            Some(tmp.path().join("failing-data")),
            Path::new("/opt/myapp"),
            config,
        )
        .expect_err("xdg-mime failed");
        unsafe {
            std::env::remove_var("NEXUM_FAKE_XDG_MIME_RC");
        }
        assert!(matches!(error, Error::Platform(_)));
        assert!(error.to_string().contains("failed with exit status: 3"));
    }
}
