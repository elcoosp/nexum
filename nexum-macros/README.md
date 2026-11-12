# nexum-macros

Attribute macros for the [nexum](https://github.com/elcoosp/nexum)
deep-linking toolkit.

## `#[deep_link]`

Removes the bootstrap ceremony: builds and validates the configuration and
runs the platform registration before `fn main` executes. Import it from
the [`nexum`](https://docs.rs/nexum) facade crate:

```rust,ignore
use nexum::deep_link;

#[deep_link(schemes = ["myapp", "beta"], app_links = ["example.com"], icon = "/icons/app.png")]
fn main() {
    // The consumer handle is bound to `_nexum_handle` (the leading
    // underscore keeps it warning-free when unused).
    if let Ok(url) = _nexum_handle.recv_blocking() {
        println!("opened {url}");
    }
}
```

| Key | Value | Effect |
| :--- | :--- | :--- |
| `schemes` | `["a", "b"]` (required) | URL schemes to register. |
| `app_links` | `["example.com"]` | App-link (universal-link style) hosts. |
| `icon` | `"/icons/app.png"` | Protocol/desktop-entry icon path. |

The generated code panics on an invalid configuration or a failed
registration; use `nexum_platform::init` directly when you need fallible
control. Applying the macro to anything but `fn main` is a compile error.

