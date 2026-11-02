//! Framework-agnostic deep linking for Rust.
//!
//! Nexum registers custom URL schemes (`myapp://…`) with the operating system
//! and delivers matching events to your application. This crate contains the
//! framework-independent building blocks:
//!
//! * [`Config`] — the schemes and app links your application handles,
//!   validated against RFC 3986 rules.
//! * [`DeepLinkHub`] — the producer side: platform code and integrations push
//!   URLs into it.
//! * [`DeepLinkHandle`] — the consumer side: blocking, async and [`Stream`]
//!   (behind the `stream` feature) access to incoming [`DeepLinkEvent`]s.
//! * [`Error`] — the single error type shared by the whole nexum stack.
//!
//! The platform-specific registration lives in [`nexum_platform`](https://docs.rs/nexum-platform),
//! and thin adapters exist for Floem, Xilem, Dioxus and GPUI.
//!
//! # Transport semantics
//!
//! The hub is a *multi-producer, multi-consumer queue*: every event is
//! delivered to **exactly one** consumer (round-robin over live receivers).
//! Clone [`DeepLinkHandle`] when you want several workers sharing the load.
//!
//! # Example
//!
//! ```
//! use nexum_core::{Config, DeepLinkHub, Source};
//!
//! let config = Config::new(["myapp"]).validate().expect("valid config");
//! let hub = DeepLinkHub::new();
//! let handle = hub.handle();
//!
//! hub.push_url("myapp://open/settings");
//! assert_eq!(handle.try_recv().as_deref(), Some("myapp://open/settings"));
//!
//! // Cold-start URLs launched from the command line carry their own source:
//! hub.push_launch_url("myapp://open/doc/42");
//! let event = handle.recv_event_blocking().expect("event");
//! assert_eq!(event.source, Source::CommandLine);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod config;
mod error;
mod event;
mod handle;
mod hub;

pub use config::{AppLink, Config, ConfigBuilder};
pub use error::{Error, Result};
pub use event::{DeepLinkEvent, Source};
pub use handle::DeepLinkHandle;
#[cfg(feature = "stream")]
pub use handle::DeepLinkStream;
pub use hub::DeepLinkHub;
