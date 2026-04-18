# Nexum

Framework-agnostic deep linking for Rust.

Nexum provides a unified API for registering custom URL schemes and receiving
deep-link events across **Windows**, **macOS**, and **Linux**. A core crate
(`nexum-core`) handles all platform specifics; thin adapter crates expose the
functionality through each framework's idioms.

## Crates

| Crate | Framework | Integration |
|-------|-----------|-------------|
| `nexum-core` | — | Blocking receiver (`std::sync::mpsc::Receiver`) |
| `nexum-gpui` | GPUI | `Global` trait + callback registry |
| `nexum-floem` | Floem | `ReadSignal<Option<String>>` (via `signal` feature) |
| `nexum-xilem` | Xilem | `task` view + `MessageProxy` |
| `nexum-dioxus` | Dioxus | `GlobalSignal` + `use_coroutine` |

## Platform Support

| Platform | Registration | Runtime Detection |
|----------|-------------|-------------------|
| Windows | Registry (per-user) | CLI arguments |
| macOS | Info.plist (manual) | Apple Events (Swift bridge) |
| Linux | `.desktop` + `xdg-mime` | CLI arguments |

## Quick Start

Add the adapter for your framework:

```toml
# For Floem
[dependencies]
nexum-floem = { version = "0.1", features = ["signal"] }
```

Then see the adapter's README and the `examples/` directory.

## Floem Integration

Enable the `signal` feature to use the high‑level `setup_signal` helper:

```rust
use nexum_floem::setup_signal;
use nexum_core::Config;

let config = Config {
    schemes: vec!["myapp".to_string()],
    app_links: vec![],
};

// Returns a reactive `ReadSignal<Option<String>>` that updates on deep links
let url_signal = setup_signal(config);
```

Under the hood, this spawns a background thread, bridges the blocking receiver
to a crossbeam channel, and uses Floem's `create_signal_from_channel` to
produce a signal that wakes the UI thread only when a URL arrives.

If you need more control (e.g., custom error handling), you can use the
lower‑level `setup` function (available without the `signal` feature) and build
your own signal.

## Building

The core crate builds on all three platforms with no external requirements:

```bash
cargo build -p nexum-core
```

Adapter crates depend on their respective UI frameworks. Build them
individually when the framework is available:

```bash
cargo build -p nexum-gpui    # requires gpui
cargo build -p nexum-floem   # requires floem
cargo build -p nexum-xilem   # requires xilem
cargo build -p nexum-dioxus  # requires dioxus
```

## Examples

| Example | Framework | Path |
|---------|-----------|------|
| `gpui-basic` | GPUI | `examples/gpui-basic/` |
| `floem-basic` | Floem | `examples/floem-basic/` |
| `xilem-basic` | Xilem | `examples/xilem-basic/` |
| `dioxus-basic` | Dioxus | `examples/dioxus-basic/` |

## License

MIT OR Apache-2.0
