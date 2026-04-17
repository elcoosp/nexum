//! Nexum Core: Framework-agnostic deep linking for Rust.

mod config;
mod error;

// Suppress warnings from old objc 0.2 macros inside platform/macos.rs
#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub mod platform;

#[cfg(not(target_os = "macos"))]
pub mod platform;

pub use config::{AppLink, Config};
pub use error::Error;

use async_channel::{Receiver, Sender};
use url::Url;

/// The main deep link manager.
pub struct Nexum {
    config: Config,
    #[allow(dead_code)]
    event_tx: Sender<Vec<Url>>,
    event_rx: Receiver<Vec<Url>>,
}

impl Nexum {
    /// Creates a new instance with the given configuration.
    pub fn new(config: Config) -> Self {
        let (event_tx, event_rx) = async_channel::unbounded();

        #[cfg(target_os = "macos")]
        platform::macos::set_event_tx(event_tx.clone());

        let nexum = Self {
            config,
            event_tx,
            event_rx,
        };

        #[cfg(target_os = "macos")]
        {
            platform::macos::setup_apple_event_listener();
            if let Some(urls) = platform::macos::get_current_urls() {
                let _ = nexum.event_tx.try_send(urls);
            }
        }

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
    pub fn get_current(&self) -> Option<Vec<Url>> {
        #[cfg(target_os = "windows")]
        return platform::windows::get_current_urls();
        #[cfg(target_os = "linux")]
        return platform::linux::get_current_urls();
        #[cfg(target_os = "macos")]
        return platform::macos::get_current_urls();
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

    /// macOS/iOS: Call this from your native app delegate when an `openURLs` event occurs.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn handle_open_urls(urls: Vec<String>) {
        platform::macos::handle_open_urls(urls);
    }
}
