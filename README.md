# Nexum

Framework-agnostic deep linking for Rust.

Nexum provides a unified API for registering custom URL schemes and receiving
deep-link events across **Windows**, **macOS**, and **Linux**. A core crate
(`nexum-core`) handles all platform specifics; thin adapter crates expose the
functionality through each framework's idioms.

## Crates

| Crate | Framework | Integration |
|-------|-----------|-------------|
| `nexum-core` | — | Async channel (`async-channel`) |
| `nexum-gpui` | GPUI (Zed) | `Global` trait + callback registry |
| `nexum-floem` | Floem | `RwSignal` |
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
# For GPUI
[dependencies]
nexum-gpui = "0.1"
```

Then see the adapter's README and the `examples/` directory.

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
