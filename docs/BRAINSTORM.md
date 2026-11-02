# Nexum Brainstorm — Performance, Modularity, DRY, KISS & Roadmap

> Status legend: ✅ implemented in this release · 🔜 planned (documented, not blocking 0.2.0)

## 1. Performance

| # | Idea | Status | Notes |
|---|------|--------|-------|
| P1 | Delete unused dependencies (`url`, `thiserror`, `once_cell`, `rust-ini`, `windows-registry`, `objc2*`, `async-channel`, `crossbeam-channel`… were declared in crates that never use them) | ✅ | Every removed dep is compile time + attack surface + supply-chain risk saved. |
| P2 | Replace `once_cell` with `std::sync::OnceLock` | ✅ | std since 1.80; one fewer third-party crate in the tree. |
| P3 | Single unbounded MPSC as the transport; deep links are rare human-driven events, so no backpressure/bounded queues are needed | ✅ | KISS: `async-channel` handles sync+async+`Stream` with one primitive. |
| P4 | Error-aware push instead of silently dropped URLs | ✅ | `nexum_platform::push_url` returns `Result` so a mis-initialized app fails loudly instead of black-holing deep links. |
| P5 | Avoid double event delivery: cold-start URLs are pushed exactly once, tagged with their source | ✅ | `Source::CommandLine` vs `Source::System` lets apps deduplicate cold-start handling. |
| P6 | Typed URL parsing happens lazily, at consumption time, never on the hot push path (UI thread) | ✅ | Push = one `String` into a channel; parsing happens in the consumer. |
| P7 | Named bridge thread for the Floem signal (`thread::Builder`) instead of anonymous `thread::spawn` | ✅ | Better crash diagnostics, negligible cost. |
| P8 | Optional `Stream` impl behind a `stream` feature (futures-core only) | ✅ | Zero cost when the feature is off. |

## 2. Modularity

| # | Idea | Status | Notes |
|---|------|--------|-------|
| M1 | `nexum-core` split into `config`, `error`, `event`, `hub`, `handle` modules with a flat re-exported facade | ✅ | Private modules + `pub use` keeps `nexum_core::Config` paths clean. |
| M2 | `nexum-platform` split into `linux`, `macos`, `windows` modules | ✅ | Each platform's glue lives in one file; the global hub/bootstrap stays in `lib.rs`. |
| M3 | One-call bootstrap `nexum_platform::init(&config) -> Result<DeepLinkHandle>` | ✅ | Replaces the copy-pasted hub/global/register dance that was duplicated in 4 adapters (DRY). |
| M4 | Workspace-level `[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]` | ✅ | Single source of truth for versions, metadata and lint policy; crates inherit via `[lints] workspace = true`. |
| M5 | Adapters re-export the core types they surface | ✅ | Users depend on one adapter crate, not two. |
| M6 | `nexum-gpui` stays a non-workspace crate tracking Zed's `main` branch | ✅ | Its git dependency is far too heavy for routine CI; it is built manually. |

## 3. DRY

| # | Idea | Status | Notes |
|---|------|--------|-------|
| D1 | `nexum_platform::init()` replaces four identical `DeepLinkHub::new() → set → register` blocks | ✅ | floem/xilem/dioxus/gpui adapters are now thin reactive shells. |
| D2 | Shared error type (`nexum_core::Error`) used by every layer | ✅ | One `Result` alias across the stack; adapters never invent their own errors. |
| D3 | Scheme filtering / CLI extraction as one pure function (`Config::extract_launch_urls`) | ✅ | Was implicit, duplicated in platform registration and impossible to test. |
| D4 | Desktop-entry writer, Exec quoting and app-name sanitizing as pure helpers | ✅ | One place to get Linux quoting right (spaces, `%` escaping, trailing `;` on MimeType). |
| D5 | Per-crate READMEs point at the same root docs; feature matrix lives once | ✅ | Crate READMEs stay short and non-duplicative. |

## 4. KISS

| # | Idea | Status | Notes |
|---|------|--------|-------|
| K1 | No `unwrap()` in cross-platform library code — errors propagate as `Result` | ✅ | Panics are only used where they are genuinely programming errors, and are documented (`# Panics`). |
| K2 | `Config` keeps plain public fields (struct-literal friendly) *plus* an opt-in builder | ✅ | No mandatory ceremony; `Config::new(["myapp"])` is enough. |
| K3 | Global hub is a plain `OnceLock` — set once, read lock-free forever | ✅ | No `Mutex<Option<Arc<…>>>` gymnastics. |
| K4 | Transport is `DeepLinkEvent { url, source }` — one struct, one enum, no trait objects | ✅ | `String` URLs stay at the edge; parsing is an explicit opt-in (`recv_url`). |
| K5 | Removed the unused `HUB_SENDER`/`HANDLE` static pair in the Dioxus adapter | ✅ | The platform hub is the single source of truth. |
| K6 | tao stays at the version dioxus-desktop requires (0.34) instead of "latest" (0.37) | ✅ | Bumping it would put two incompatible `tao` types in the same binary — correctness beats version vanity. |

## 5. Ambitious complementary features

| # | Idea | Status | Notes |
|---|------|--------|-------|
| F1 | **Typed deep-link events** — `DeepLinkEvent` carries the raw URL plus a `Source` (cold-start CLI vs. runtime OS delivery) | ✅ | Lets apps skip redundant "first launch" handling. |
| F2 | **URL parsing helpers** — `recv_url()` / `DeepLinkEvent::parse_url()` return `url::Url` with a proper `MalformedUrl` error | ✅ | Previously URLs were opaque strings end-to-end. |
| F3 | **Config validation** — RFC 3986 scheme grammar, non-empty scheme list, app-link hosts | ✅ | Fails at startup with precise messages instead of mysterious OS-level failures. |
| F4 | **Builder API** — `Config::builder().scheme("myapp").app_link(..).build()?` | ✅ | Validated construction for API-heavy call sites. |
| F5 | **All-args launch-URL extraction** — every `argv` entry that parses to a *registered* scheme is picked up (not just `args[1]`) | ✅ | Handles multi-URL hand-offs and ignores unrelated flags. |
| F6 | **Windows: configurable protocol icon** — `Config::icon_path` drives `DefaultIcon` | ✅ | Falls back to the running executable. |
| F7 | **Linux: proper desktop entries** — quoted `Exec`, `%%` escaping, optional `Icon=`, trailing-`;` MimeType list, sanitized app name, `xdg-mime` failures reported | ✅ | Registration is now verifiable and non-lossy. |
| F8 | **`Stream` support** — `DeepLinkHandle::into_stream()` behind the `stream` feature | ✅ | Composable with any async runtime / `tokio_stream` / `futures`. |
| F9 | **Headless-testable adapters** — relay loops extracted into pure async functions; xilem relay tested against a mock `RawProxy` | ✅ | UI-framework glue is no longer a coverage black hole. |
| F10 | **Single-instance hand-off** (second app instance forwards its URL to the running instance over a local socket and exits) | 🔜 | Needs a small IPC crate (`interprocess`) + per-platform naming; substantial, deserves its own RFC. |
| F11 | **macOS `Info.plist` validator** (warn when registered schemes are missing from the bundle) | 🔜 | Requires `NSBundle` plumbing; useful only inside a bundle. |
| F12 | **`nexum-macros`** — `#[nexum::deep_link]` attribute generating config + bootstrap | 🔜 | A proc-macro crate would roughly double cold compile time for little gain today. |
| F13 | **Android/iOS backends** (intent filters / universal links) | 🔜 | Blocked on the UI frameworks themselves. |
| F14 | **Bounded-queue mode** (`hub_config: Bounded(usize)`) for apps that must not buffer unbounded URLs | 🔜 | Current unbounded queue matches the event rate; revisit if a use case appears. |

## 6. Quality infrastructure (part of the same brainstorm)

| # | Idea | Status | Notes |
|---|------|--------|-------|
| Q1 | `#![forbid(unsafe_code)]` in `nexum-core` | ✅ | The only `unsafe` lives in platform glue where it is unavoidable (objc2). |
| Q2 | Clippy **pedantic** enabled workspace-wide with **zero `#[allow]`/`#[expect]` suppressions** | ✅ | Enforced in CI with `-D warnings`. |
| Q3 | `missing_docs` + `rust_2018_idioms` warnings; every public item documented | ✅ | Docs run through the same `-D warnings` gate. |
| Q4 | 100% line coverage gate for `nexum-core`, `nexum-platform`, `nexum-floem` (default features) via `cargo-llvm-cov --fail-under-lines 100` | ✅ | `nexum-xilem` reaches 100% too (mocked proxy); `nexum-dioxus` covers everything except the event-loop-bound `launch_desktop` — see `docs/COVERAGE.md`. |
| Q5 | CI matrix: Linux (full workspace), macOS, Windows + fmt + clippy + MSRV + coverage | ✅ | `.github/workflows/ci.yml`. |
| Q6 | Release automation: git-cliff changelog, GitHub Releases, crates.io publish in dependency order | ✅ | `.github/workflows/release.yml` + `cliff.toml`. |
| Q7 | `cargo-deny` (licenses/advisories) and `typos` checks | ✅ | `.github/workflows/security.yml`, `deny.toml`, `.typos.toml`. |
| Q8 | Dependabot for Cargo + Actions | ✅ | `.github/dependabot.yml`. |
| Q9 | Dual MIT / Apache-2.0 license files (required for crates.io `MIT OR Apache-2.0` metadata) | ✅ | `LICENSE-MIT`, `LICENSE-APACHE`. |
