//! <div align="center">
//! <strong>Framework-agnostic deep linking for Rust</strong><br/>
//! One dependency, one attribute, registered URL schemes on Windows,
//! macOS and Linux.
//! </div>
//!
//! This is the facade crate of the nexum toolkit: it re-exports the core
//! primitives, the platform bootstrap and the [`deep_link`] attribute macro,
//! so a framework-less application needs exactly one dependency.
//!
//! ```no_run
//! use nexum::deep_link;
//!
//! #[deep_link(schemes = ["myapp"])]
//! fn main() {
//!     // Deep links are already registered. `_nexum_handle` is the
//!     // consumer handle; the global hub also exposes `nexum::platform`.
//!     if let Ok(url) = _nexum_handle.recv_blocking() {
//!         println!("opened {url}");
//!     }
//! }
//! ```
//!
//! UI-framework users should prefer the dedicated adapter crates
//! ([`nexum-floem`], [`nexum-xilem`], [`nexum-dioxus`], [`nexum-gpui`]),
//! which bridge deep links into their framework's reactive model; they
//! re-export everything in here as well.
//!
//! [`nexum-floem`]: https://docs.rs/nexum-floem
//! [`nexum-xilem`]: https://docs.rs/nexum-xilem
//! [`nexum-dioxus`]: https://docs.rs/nexum-dioxus
//! [`nexum-gpui`]: https://docs.rs/nexum-gpui

#[cfg(feature = "stream")]
pub use nexum_core::DeepLinkStream;
pub use nexum_core::{
    AppLink, Config, DeepLinkEvent, DeepLinkHandle, DeepLinkHub, Error, Result, Source,
};
pub use nexum_macros::deep_link;
pub use nexum_platform as platform;
pub use nexum_platform::{hub, init, init_with, push_launch_url, push_url, register, set_hub};

#[cfg(target_os = "macos")]
pub use nexum_platform::validate_bundle;
