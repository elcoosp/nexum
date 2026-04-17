//! Nexum Core: Framework-agnostic deep linking for Rust.
//!
//! This crate handles platform-specific URL scheme registration and provides
//! an async channel for receiving incoming URLs. Framework adapters should
//! wrap [`Nexum`] and deliver events using the framework's idioms.
//!
//! # Platform Support
//!
//! | Platform | Registration Method    | URL Detection Method |
//! |----------|------------------------|----------------------|
//! | Windows  | Windows Registry       | CLI arguments        |
//! | macOS    | Info.plist (manual)    | Apple Events (bridge)|
//! | Linux    | `.desktop` file + xdg  | CLI arguments        |
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use nexum_core::{Config, Nexum};
//!
//! let config = Config {
//!     schemes: vec!["myapp".to_string()],
//!     app_links: vec![],
//! };
//!
//! let nexum = Nexum::new(config);
//! nexum.register_all().expect("registration failed");
//!
//! // Spawn a listener task
//! let rx = nexum.event_receiver();
//! // In an async context: while let Ok(urls) = rx.recv().await { ... }
//!
//! // Or check for a URL passed at launch
//! if let Some(urls) = nexum.get_current() {
//!     println!("Launched with: {:?}", urls);
//! }
//! ```

mod config;
mod error;
mod platform;

pub use config::{AppLink, Config};
pub use error::Error;

use async_channel::{Receiver, Sender};
use url::Url;

/// The main deep link manager.
///
/// Holds configuration, an async event channel, and exposes platform-specific
/// registration / detection methods. Framework adapters wrap this type.
pub struct Nexum {
    config: Config,
    event_tx: Sender<Vec<Url>>,
    event_rx: Receiver<Vec<Url>>,
}

impl Nexum {
    /// Creates a new instance with the given configuration.
    ///
    /// On platforms that support it (Windows, Linux), this immediately checks
    /// for a URL passed via command-line arguments and sends it through the
    /// event channel.
    pub fn new(config: Config) -> Self {
        let (event_tx, event_rx) = async_channel::unbounded();

        #[cfg(target_os = "macos")]
        platform::macos::set_event_tx(event_tx.clone());

        let nexum = Self {
            config,
            event_tx,
            event_rx,
        };

        #[cfg(target_os = "windows")]
        if let Some(urls) = platform::windows::get_current_urls() {
            let _ = nexum.event_tx.try_send(urls);
        }
        #[cfg(target_os = "linux")]
        if let Some(urls) = platform::linux::get_current_urls() {
            let _ = nexum.event_tx.try_send(urls);
        }

        nexum
    }

    /// Registers all schemes defined in the configuration with the OS.
    ///
    /// - **Windows**: Writes to `HKEY_CURRENT_USER\Software\Classes`.
    /// - **macOS**: No-op; see [README](../nexum-core/README.md) for Info.plist instructions.
    /// - **Linux**: Creates/updates a `.desktop` file and runs `xdg-mime default`.
    pub fn register_all(&self) -> Result<(), Error> {
        #[cfg(target_os = "windows")]
        platform::windows::register_schemes(&self.config)?;
        #[cfg(target_os = "macos")]
        platform::macos::register_schemes(&self.config)?;
        #[cfg(target_os = "linux")]
        platform::linux::register_schemes(&self.config)?;
        Ok(())
    }

    /// Returns a clone of the event receiver that streams incoming URLs.
    ///
    /// Multiple callers may each hold a receiver clone; all will receive
    /// the same deep-link events.
    pub fn event_receiver(&self) -> Receiver<Vec<Url>> {
        self.event_rx.clone()
    }

    /// Returns the current deep link URLs, if any.
    ///
    /// On Windows and Linux this checks command-line arguments.
    /// On macOS this returns `None` (URLs arrive via the event channel after
    /// [`handle_open_urls`](Nexum::handle_open_urls) is called).
    pub fn get_current(&self) -> Option<Vec<Url>> {
        #[cfg(target_os = "windows")]
        return platform::windows::get_current_urls();
        #[cfg(target_os = "linux")]
        return platform::linux::get_current_urls();
        #[allow(unreachable_code)]
        None
    }

    /// Checks if a scheme is registered as the default handler.
    ///
    /// Always returns [`Error::UnsupportedPlatform`] on macOS (registration
    /// is declarative via Info.plist).
    pub fn is_registered(&self, scheme: &str) -> Result<bool, Error> {
        #[cfg(target_os = "windows")]
        return platform::windows::is_registered(scheme);
        #[cfg(target_os = "linux")]
        return platform::linux::is_registered(scheme);
        #[allow(unused_variables)]
        Err(Error::UnsupportedPlatform)
    }

    /// Unregisters a scheme (Windows/Linux only).
    pub fn unregister(&self, scheme: &str) -> Result<(), Error> {
        #[cfg(target_os = "windows")]
        platform::windows::unregister_scheme(scheme)?;
        #[cfg(target_os = "linux")]
        platform::linux::unregister_scheme(scheme)?;
        #[allow(unused_variables)]
        let _ = scheme;
        #[allow(unreachable_code)]
        Ok(())
    }

    /// macOS/iOS: Call this from your native app delegate when an `openURLs`
    /// event occurs. The URLs will be forwarded through the event channel.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn handle_open_urls(urls: Vec<String>) {
        platform::macos::handle_open_urls(urls);
    }
}
