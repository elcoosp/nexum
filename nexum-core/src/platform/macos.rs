use crate::{Config, Error};
use std::sync::OnceLock;
use url::Url;

static EVENT_TX: OnceLock<async_channel::Sender<Vec<Url>>> = OnceLock::new();

/// Stores the event sender for use by the Apple Event handler.
#[allow(dead_code)]
pub fn set_event_tx(tx: async_channel::Sender<Vec<Url>>) {
    EVENT_TX.set(tx).ok();
}

/// Registration on macOS is a no-op; users must manually edit Info.plist.
pub fn register_schemes(config: &Config) -> Result<(), Error> {
    if !config.schemes.is_empty() {
        eprintln!(
            "Nexum: Deep link schemes must be manually added to Info.plist. Schemes: {:?}",
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

/// Checks CLI arguments for a URL (useful for testing via `cargo run -- myapp://...`)
pub fn get_current_urls() -> Option<Vec<Url>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        if let Ok(url) = Url::parse(&args[1]) {
            return Some(vec![url]);
        }
    }
    None
}
