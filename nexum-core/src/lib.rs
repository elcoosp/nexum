//! Nexum Core: Framework-agnostic deep linking for Rust.
//!
//! This crate handles platform-specific URL scheme registration and provides
//! an async channel for receiving incoming URLs. Framework adapters should
//! wrap `Nexum` and deliver events using the framework's idioms.

mod config;
mod error;
mod platform;

pub use config::{AppLink, Config};
pub use error::Error;

use async_channel::{Receiver, Sender};
use url::Url;

/// The main deep link manager.
pub struct Nexum {
    config: Config,
    event_tx: Sender<Vec<Url>>,
    event_rx: Receiver<Vec<Url>>,
}

impl Nexum {
    /// Creates a new instance with the given configuration.
    /// On platforms that support it, this immediately checks for a current URL
    /// (e.g., from command line arguments) and sends it through the channel.
    pub fn new(config: Config) -> Self {
        let (event_tx, event_rx) = async_channel::unbounded();
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
    pub fn register_all(&self) -> Result<(), Error> {
        #[cfg(target_os = "windows")]
        platform::windows::register_schemes(&self.config)?;
        #[cfg(target_os = "macos")]
        platform::macos::register_schemes(&self.config)?;
        #[cfg(target_os = "linux")]
        platform::linux::register_schemes(&self.config)?;
        Ok(())
    }

    /// Returns a receiver that streams incoming URLs.
    pub fn event_receiver(&self) -> Receiver<Vec<Url>> {
        self.event_rx.clone()
    }

    /// Returns the current deep link URLs, if any.
    /// On Windows and Linux, this checks command line arguments.
    pub fn get_current(&self) -> Option<Vec<Url>> {
        #[cfg(target_os = "windows")]
        return platform::windows::get_current_urls();
        #[cfg(target_os = "linux")]
        return platform::linux::get_current_urls();
        #[cfg(target_os = "macos")]
        return None;
    }

    /// Checks if a scheme is registered as the default handler.
    pub fn is_registered(&self, _scheme: &str) -> Result<bool, Error> {
        #[cfg(target_os = "windows")]
        return platform::windows::is_registered(_scheme);
        #[cfg(target_os = "linux")]
        return platform::linux::is_registered(_scheme);
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        Err(Error::UnsupportedPlatform)
    }

    /// Unregisters a scheme (Windows/Linux only).
    pub fn unregister(&self, _scheme: &str) -> Result<(), Error> {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            #[cfg(target_os = "windows")]
            platform::windows::unregister_scheme(_scheme)?;
            #[cfg(target_os = "linux")]
            platform::linux::unregister_scheme(_scheme)?;
            Ok(())
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        Err(Error::UnsupportedPlatform)
    }

    /// Returns a clone of the sender for platform-specific use (e.g., macOS Apple Events).
    pub fn event_sender(&self) -> Sender<Vec<Url>> {
        self.event_tx.clone()
    }

    /// macOS/iOS: Call this from your native app delegate when an `openURLs` event occurs.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn handle_open_urls(urls: Vec<String>) {
        platform::macos::handle_open_urls(urls);
    }
}
