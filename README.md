<div align="center">
  <img src="docs/logo.png" alt="Nexum Logo" width="300"/>
  <p>
    <strong>Framework-agnostic deep linking for Rust</strong><br/>
    A unified API for registering custom URL schemes and receiving deep-link events across Windows, macOS, and Linux.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.85%2B%20%7C%202024%20edition-000000?style=flat-square&logo=rust" alt="Rust"/>
    <img src="https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue?style=flat-square" alt="License"/>
    <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-262D3A?style=flat-square" alt="Platforms"/>
    <img src="https://img.shields.io/badge/Ecosystem-GPUI%20%7C%20Floem%20%7C%20Xilem%20%7C%20Dioxus-6A0DAD?style=flat-square" alt="Ecosystem"/>
  </p>
</div>

---

Nexum provides a unified, cross-platform solution for handling custom URL schemes and deep-link events in Rust applications. `nexum-core` holds the framework-independent primitives, `nexum-platform` performs the OS-level registration, and thin adapter crates expose everything through the idioms of your favorite UI framework.

## Ecosystem

Nexum is organized as a modular workspace to keep dependencies minimal and integrations seamless.

| Crate | Framework | Integration Mechanism |
| :--- | :--- | :--- |
| [`nexum-core`](nexum-core/) | *None* | `DeepLinkHub` producer + `DeepLinkHandle` consumer (blocking, async, `Stream`) |
| [`nexum-platform`](nexum-platform/) | *None* | `init` / `register` — registry, Apple events, `.desktop` + `xdg-mime` |
| [`nexum-floem`](nexum-floem/) | Floem | `setup` + `signal` → `ReadSignal<Option<String>>` (via `signal` feature) |
| [`nexum-xilem`](nexum-xilem/) | Xilem | `setup` + `with_deep_links` view wrapper (`task_raw` + `MessageProxy`) |
| [`nexum-dioxus`](nexum-dioxus/) | Dioxus | `launch_desktop` one-call path + `use_deep_link` signal |
| [`nexum-gpui`](nexum-gpui/) | GPUI | `setup_deep_links` + `attach_deep_link` (Zed git registry, outside the workspace) |

## Platform Support

| Platform | Registration Method | Runtime Detection |
| :--- | :--- | :--- |
| **Windows** | Registry (per-user) | CLI arguments |
| **macOS** | `Info.plist` (manual setup) | Apple Events (`GURL` via `objc2`) |
| **Linux** | `.desktop` file + `xdg-mime` | CLI arguments |

## Quick Start

Add the adapter for your chosen framework to your `Cargo.toml`. For example, with Floem:

```toml
[dependencies]
nexum-floem = { version = "0.2", features = ["signal"] }
```

Then, initialize the deep-link handler in your application:

```rust,ignore
use floem::prelude::*;
use nexum_core::Config;
use nexum_floem::signal;

let config = Config::builder()
    .scheme("myapp")
    .build()
    .expect("valid config");

// Returns a reactive `ReadSignal<Option<String>>` that updates on deep links
let url_signal = signal(&config);
```

> [!NOTE]
> **Under the Hood**
> The `signal` helper spawns a background thread, bridges the hub into Floem's reactive runtime through `create_signal_from_channel`, and returns a `ReadSignal<Option<String>>`. This ensures the UI thread only wakes when a new URL arrives.
> For more control (e.g., custom consumption on your own thread), use the lower-level `setup` function (available without the `signal` feature) which returns the raw `DeepLinkHandle`.

Using `nexum-core` directly? The same two-sided shape applies everywhere:

```rust
use nexum_core::{Config, DeepLinkHub, Source};

let config = Config::builder().scheme("myapp").build().expect("valid config");
let hub = DeepLinkHub::new();
let handle = hub.handle();

hub.push_url("myapp://open/42");
let event = handle.recv_event_blocking().expect("event");
assert_eq!(event.source, Source::System);
```

## Building

The core crates build on all three platforms with no external dependencies:

```bash
cargo build -p nexum-core -p nexum-platform
```

Adapter crates depend on their respective UI frameworks. Build them individually when the framework environment is available:

```bash
cargo build -p nexum-gpui    # requires the Zed git registry (outside the workspace)
cargo build -p nexum-floem   # requires floem
cargo build -p nexum-xilem   # requires xilem
cargo build -p nexum-dioxus  # requires dioxus
```

## Examples

Explore ready-to-run implementations for each supported framework in the `examples/` directory:

| Example | Framework | Path |
| :--- | :--- | :--- |
| `gpui-basic` | GPUI | `examples/gpui-basic/` |
| `floem-basic` | Floem | `examples/floem-basic/` |
| `xilem-basic` | Xilem | `examples/xilem-basic/` |
| `dioxus-basic` | Dioxus | `examples/dioxus-basic/` |

## Quality & CI

- **Tests:** the workspace keeps 100 % line coverage on `nexum-core`, `nexum-platform` and `nexum-floem` (enforced with `cargo-llvm-cov --fail-under-lines 100`, see [`docs/COVERAGE.md`](docs/COVERAGE.md)).
- **Linting:** `clippy --pedantic` with `unsafe_code = "forbid"` and `missing_docs` — zero warnings, zero allows.
- **CI/CD:** [`.github/workflows/ci.yml`](.github/workflows/ci.yml) (fmt, clippy, 3-OS test matrix, MSRV, docs, coverage), [`release.yml`](.github/workflows/release.yml) (git-cliff changelog + crates.io publishing), [`security.yml`](.github/workflows/security.yml) (cargo-deny + typos), Dependabot for weekly updates.

> [!TIP]
> For detailed setup instructions specific to your framework (especially macOS `Info.plist` configuration), refer to the individual adapter crate's README or the provided examples.
