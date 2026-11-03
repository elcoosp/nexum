# nexum-leptos

Leptos adapter for nexum deep linking. Two entry points are available:

- `setup` — registers the schemes and returns a `DeepLinkHandle` for manual
  (blocking or async) consumption. Always available.
- `signal` — bridges deep links into a Leptos
  `RwSignal<Option<String>>`. Available with the `signal` feature.
- `signal_from_handle` — the framework-agnostic core of the same bridge,
  usable everywhere Leptos compiles (including `wasm32` in the browser)
  whenever a handle already exists.

## Usage (signal feature)

```toml
[dependencies]
nexum-leptos = { version = "0.3", features = ["signal"] }
```

```rust,ignore
use leptos::prelude::*;
use nexum_core::Config;
use nexum_leptos::signal;

let config = Config::builder().scheme("myapp").build().expect("valid config");

// Updates whenever a deep link arrives.
let url = signal(&config);

view! {
    <p>{move || url.get().unwrap_or_else(|| "waiting…".to_owned())}</p>
}
```

On `wasm32` (browser builds) the OS cannot register schemes for the page, so
the registration is performed by the embedding platform. Use the bridge
directly:

```rust,ignore
let hub = DeepLinkHub::new();
let handle = hub.handle();
// What the OS layer feeds in on desktop, the embedder feeds here:
hub.push_url("myapp://open/settings");
let url = nexum_leptos::signal_from_handle(&handle);
```

## Usage (manual)

```rust,ignore
use nexum_core::Config;
use nexum_leptos::setup;

let config = Config::builder().scheme("myapp").build().expect("valid config");
let handle = setup(&config).expect("registration failed");

// Consume on your own thread/loop:
while let Ok(url) = handle.recv_blocking() {
    println!("opened {url}");
}
```

## How the bridge works

`signal` / `signal_from_handle` spawn the `relay_to_signal` future through
`leptos::task::spawn_local`; each incoming URL is written into an
`RwSignal<Option<String>>`, so fine-grained effects update the DOM without
any virtual DOM overhead. The executor must already be initialized — Leptos
does this in `mount_to_body` on the web and through its SSR runtime on the
server.

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
