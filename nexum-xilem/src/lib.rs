use async_channel::Receiver;
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// Creates a new Nexum instance, registers all schemes, and returns a receiver
/// that streams incoming deep link URLs.
pub fn create_deep_link_listener(config: Config) -> Receiver<Vec<Url>> {
    let inner = CoreNexum::new(config);
    inner.register_all().expect("Failed to register schemes");
    inner.event_receiver()
}

/// Returns the current deep link URLs, if any.
pub fn get_current(config: &Config) -> Option<Vec<Url>> {
    CoreNexum::new(config.clone()).get_current()
}

/// Checks if a scheme is registered.
pub fn is_registered(config: &Config, scheme: &str) -> Result<bool, nexum_core::Error> {
    CoreNexum::new(config.clone()).is_registered(scheme)
}

/// Unregisters a scheme (Windows/Linux only).
pub fn unregister(config: &Config, scheme: &str) -> Result<(), nexum_core::Error> {
    CoreNexum::new(config.clone()).unregister(scheme)
}

/// macOS/iOS: Call this from your native app delegate.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn handle_open_urls(urls: Vec<String>) {
    CoreNexum::handle_open_urls(urls);
}
