# nexum-floem

Floem adapter for nexum deep linking. Two entry points are available:

- `setup` — registers the schemes and returns a `DeepLinkHandle` for manual
  (blocking or async) consumption. Always available.
- `signal` — bridges deep links into a Floem
  `ReadSignal<Option<String>>`. Available with the `signal` feature.

## Usage (signal feature)

```toml
[dependencies]
nexum-floem = { version = "0.2", features = ["signal"] }
```

```rust,ignore
use floem::prelude::*;
use nexum_core::Config;
use nexum_floem::signal;

let config = Config::builder().scheme("myapp").build().expect("valid config");

// Updates whenever a deep link arrives.
let url = signal(&config);

let app = floem::new_app(move || {
    floem::views::label(move || url.get().unwrap_or_else(|| "waiting…".into()))
});
floem::launch(app);
```

## Usage (manual)

```rust,ignore
use nexum_core::Config;
use nexum_floem::setup;

let config = Config::builder().scheme("myapp").build().expect("valid config");
let handle = setup(&config).expect("registration failed");

// Consume on your own thread/loop:
while let Ok(url) = handle.recv_blocking() {
    println!("opened {url}");
}
```

## How the bridge works

`signal` spawns a background thread that forwards events from the
`DeepLinkHandle` into Floem's reactive runtime through
`create_signal_from_channel`; the UI thread only wakes when a URL arrives.

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
