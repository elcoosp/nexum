# nexum-core

Framework-agnostic deep-linking primitives for Rust. Nexum registers custom
URL schemes (`myapp://…`) with the operating system and delivers matching
events to your application; this crate contains the platform-independent
building blocks every other nexum crate is built on.

## What's inside

| Type | Role |
| :--- | :--- |
| `Config` / `ConfigBuilder` | The schemes and app links your application handles, validated against RFC 3986 rules. |
| `DeepLinkHub` | The producer side — platform code and adapters push URLs into it. |
| `DeepLinkHandle` | The consumer side — blocking, async and `Stream` access to incoming events. |
| `DeepLinkEvent` / `Source` | One delivered URL plus its origin (`System`, `CommandLine`, …). |
| `Error` / `Result` | The crate error type (`thiserror`). |

## Usage

```rust
use nexum_core::{Config, DeepLinkHub, Source};

// Build and validate a configuration.
let config = Config::builder()
    .scheme("myapp")
    .build()
    .expect("valid config");

// Create a hub: producers push, consumers receive.
let hub = DeepLinkHub::new();
let handle = hub.handle();

hub.push_url("myapp://open/42");
hub.push_launch_url("myapp://cold-start");

let event = handle.recv_event_blocking().expect("event");
assert_eq!(event.source, Source::System);
```

## Features

| Feature | Default | Description |
| :--- | :--- | :--- |
| `stream` | ✔ | `DeepLinkStream`: consume events as a `futures_core::Stream`. |

Without `stream` the handle exposes blocking (`recv`, `try_recv`, `drain`)
and async (`recv_async`, `recv_event_async`) APIs only.

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
