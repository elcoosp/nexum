use crate::{Config, Error};
use std::sync::OnceLock;
use url::Url;

// TODO: When implementing native Apple Event handling, add the `objc` crate
// dependency and use it to register an event handler for `AEGetParamDesc`
// to intercept `openURLs` events directly from Rust without Swift bridging.

static EVENT_TX: OnceLock<async_channel::Sender<Vec<Url>>> = OnceLock::new();

/// Stores the event sender for use by the Apple Event handler.
pub fn set_event_tx(tx: async_channel::Sender<Vec<Url>>) {
    let _ = EVENT_TX.set(tx);
}

/// Registration on macOS is a no-op; users must manually edit Info.plist.
/// See nexum-core/README.md for setup instructions.
pub fn register_schemes(config: &Config) -> Result<(), Error> {
    if !config.schemes.is_empty() {
        eprintln!(
            "[nexum-core] Deep link schemes must be manually added to Info.plist. Schemes: {:?}",
            config.schemes
        );
    }
    Ok(())
}

/// Call this from your AppDelegate's `application:openURLs:` method.
pub fn handle_open_urls(url_strings: Vec<String>) {
    let urls: Vec<Url> = url_strings
        .into_iter()
        .filter_map(|s| Url::parse(&s).ok())
        .collect();
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.try_send(urls);
    }
}
