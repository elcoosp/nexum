# Coverage strategy

`nexum` enforces **100 % line coverage** for the logic crates in CI
(`nexum-core`, `nexum-platform`, `nexum-floem` — see the `coverage` job in
`.github/workflows/ci.yml`):

```bash
cargo llvm-cov --package nexum-core --package nexum-platform \
  --package nexum-floem --all-features --fail-under-lines 100
```

The full workspace additionally publishes an unrestricted coverage report to
Codecov, which includes the remaining adapters and examples.

## Why a per-crate gate instead of a workspace gate

The adapter crates wrap UI frameworks whose entry points are bound to a real
windowing event loop:

| Crate        | Coverage policy | Reason |
|--------------|-----------------|--------|
| `nexum-core`    | 100 % enforced | Pure primitives — config, hub (incl. bounded mode), handles, events. |
| `nexum-platform`| 100 % enforced | OS registration, single-instance hand-off, plist validator and mobile generators, all isolated behind explicit inputs and a fake `xdg-mime`. |
| `nexum-floem`   | 100 % enforced | Thin adapter; the signal bridge is factored into a testable `relay_urls` function. |
| `nexum-xilem`   | report-only    | `task_raw` + `fork` glue only runs inside a live Xilem runtime. |
| `nexum-dioxus`  | report-only    | `with_custom_event_handler` is invoked by the tao event loop inside `launch_desktop`. |
| `nexum-gpui`    | report-only    | Not part of the workspace build (GPUI requires the Zed git registry); callback glue is event-loop bound. |
| `nexum-macros`  | report-only    | Proc-macro crates compile as host dylibs, which `cargo-llvm-cov` cannot instrument; the expansion logic (`deep_link_impl`) is nonetheless fully covered by unit tests over `proc-macro2` tokens. |

Everything *extractable* from those event-loop-bound glue functions is
extracted into plain functions and covered — see the next section.

## Techniques used to keep the gate honest

1. **No suppression attributes.** The workspace contains no
   `#[coverage(off)]` / `#[cfg_attr(coverage, ...)]` pragmas; instead any
   hard-to-reach glue is refactored into a pure function that tests drive
   directly.

2. **One serial integration test for process-global state.**
   `nexum-platform`'s `OnceLock` hub and the mutated environment
   (`XDG_DATA_HOME`, `PATH`, fake `xdg-mime`) would race under the default
   parallel test runner, so every global-state scenario lives inside a
   single `#[test]` function that runs the steps in a fixed order. The
   single-instance hand-off lifecycle (primary acquisition, hand-off over
   a real socket, stale-socket detection, guard-drop cleanup) is part of
   that serial test for the same reason.

3. **Explicit inputs over hidden environment access.**
   `linux::write_registration(Option<PathBuf>, &Path, &Config)` accepts the
   data directory and executable as parameters, so tests reach:
   - the missing XDG data directory error (`None`),
   - the `nexum-app` fallback for executables whose name sanitizes to
     nothing (`Path::new("###")`),
   - the `xdg-mime` failure branch (the fake tool honours
     `NEXUM_FAKE_XDG_MIME_RC`).

4. **In-memory I/O for protocol code.** The single-instance wire protocol
   (`encode_request` / `parse_header` / `serve` / `exchange`) is generic
   over `Read + Write`, so tests use `std::io::Cursor` buffers; `serve`
   additionally takes the URL delivery as a closure, which keeps the tests
   free of the global hub.

5. **Testable relays instead of closures in threads.** Each adapter keeps
   its runtime glue (channels, proxies, event handlers) in a function that
   takes the incoming URLs and pushes them into the hub
   (`relay_cold_start_urls` in `nexum-platform`,
   `relay_urls` in `nexum-floem` / `nexum-xilem`,
   `relay_to_callback` in `nexum-dioxus`). The thread/event-loop wrapper
   that calls the relay is the only uncovered line — and it is a one-line
   delegation.

6. **Platform code is cross-checked even where it cannot run.** The
   `cfg`-gated backends are type-checked on every relevant target from the
   Linux CI host — `cargo check --target x86_64-apple-darwin` (macOS
   NSBundle validator), `--target x86_64-pc-windows-gnu` (registry
   backend), `--target aarch64-linux-android` and
   `--target aarch64-apple-ios` (mobile intake) — so typos cannot hide
   behind `#[cfg]`.

7. **Doctests count.** `cargo llvm-cov` runs doctests too; every public API
   example in the docs is therefore executed (or `no_run`-compiled) in the
   coverage build.

## Reproducing locally

```bash
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview

# Gate (must exit 0):
cargo llvm-cov --package nexum-core --package nexum-platform \
  --package nexum-floem --all-features --fail-under-lines 100 --summary-only

# Per-file detail, including the exact missing lines:
cargo llvm-cov --package nexum-core --all-features --lcov --output-path lcov.info
```
