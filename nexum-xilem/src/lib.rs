//! Nexum Xilem adapter: deep linking integration for the Xilem UI framework.
//!
//! Provides a [`deep_link_task`] function that returns a Xilem `task` view.
//! The task spawns an async listener and forwards incoming URLs as messages
//! through a [`MessageProxy`], which Xilem routes to your [`App::update`]
//! method.
//!
//! # Example
//!
//! ```rust,no_run
//! use xilem::{view::flex, App, AppLauncher, WidgetView};
//! use nexum_xilem::deep_link_task;
//! use nexum_core::Config;
//! use url::Url;
//!
//! #[derive(Debug)]
//! enum Msg { DeepLink(Vec<Url>) }
//!
//! struct State { last: Option<Vec<Url>> }
//!
//! impl App for State {
//!     type Message = Msg;
//!     fn update(&mut self, msg: Msg) {
//!         if let Msg::DeepLink(urls) = msg { self.last = Some(urls); }
//!     }
//!     fn view(&mut self) -> xilem::WidgetView<Msg> {
//!         let config = Config {
//!             schemes: vec!["myapp".to_string()],
//!             app_links: vec![],
//!         };
//!         flex((
//!             xilem::view::label("Hello".to_string()),
//!             deep_link_task(config, Msg::DeepLink),
//!         ))
//!     }
//! }
//!
//! fn main() { AppLauncher::new(State { last: None }).run(); }
//! ```

use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;
use xilem::view::task;
use xilem::view::WidgetView;

/// Creates a Xilem `task` view that spawns a deep link listener and sends
/// each batch of URLs as a message through the provided [`MessageProxy`].
///
/// `make_message` converts a `Vec<Url>` into your application's message type.
pub fn deep_link_task<M, F>(config: Config, make_message: F) -> impl WidgetView<M>
where
    M: Send + 'static,
    F: Fn(Vec<Url>) -> M + Send + 'static,
{
    task(
        |proxy| async move {
            let inner = CoreNexum::new(config);
            if let Err(e) = inner.register_all() {
                eprintln!("[nexum-xilem] Failed to register schemes: {}", e);
            }
            let rx = inner.event_receiver();
            while let Ok(urls) = rx.recv().await {
                let msg = make_message(urls);
                let _ = proxy.send(msg).await;
            }
        },
        |_state, _proxy| {},
    )
}

/// Returns the current deep link URLs from CLI arguments, if any (Windows/Linux).
pub fn get_current(config: &Config) -> Option<Vec<Url>> {
    CoreNexum::new(config.clone()).get_current()
}

/// Checks if a scheme is registered as the default handler.
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
