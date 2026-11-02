# nexum

Framework-agnostic deep linking for Rust — one dependency, one bootstrap.
The `nexum` crate bundles the core primitives (`nexum-core`), the
OS-level registration (`nexum-platform`) and the `#[deep_link]` attribute
macro into a single dependency for framework-less applications.

## Usage

```rust,ignore
use nexum::deep_link;

#[deep_link(schemes = ["myapp"], icon = "/icons/app.png")]
fn main() {
    // Deep links are registered before the body runs. The consumer
    // handle is bound to `_nexum_handle`.
    if let Ok(url) = _nexum_handle.recv_blocking() {
        println!("opened {url}");
    }
}
```

The macro accepts:

| Key | Value | Effect |
| :--- | :--- | :--- |
| `schemes` | `["a", "b"]` (required) | URL schemes to register. |
| `app_links` | `["example.com"]` | App-link (universal-link style) hosts. |
| `icon` | `"/icons/app.png"` | Protocol/desktop-entry icon path. |

Invalid configurations and failed registrations panic with a precise
message — both are unrecoverable startup errors. For fallible control use
`nexum::platform::init` directly.

## Without the macro

```rust,ignore
use nexum::{Config, init};

let config = Config::builder().scheme("myapp").build().expect("valid config");
let handle = init(&config).expect("registration failed");
```

## Feature overview

- `nexum::platform::single_instance` (feature `single-instance`) — forward
  URLs from a second launch to the running instance over a local socket.
- `nexum::platform::plist` + `nexum::validate_bundle` (macOS) — verify the
  bundle declares every scheme.
- `nexum::platform::mobile` — Android `intent-filter` / iOS `Info.plist`
  generators plus the runtime intake entry points.
- Bounded hubs: `init_with(&config, DeepLinkHub::bounded(16))`.

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
