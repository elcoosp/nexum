I'm using the writing-plans skill to create the implementation plan.

# Nexum: Framework‑Agnostic Deep Linking for Rust – Revised Complete Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a framework‑agnostic deep‑linking core crate (`nexum-core`) and adapters for GPUI, Floem, Xilem, and Dioxus that enable Rust applications to register custom URL schemes and receive incoming URLs across Windows, macOS, and Linux.

**Architecture:** The core crate contains platform‑specific registration and URL detection, exposing an async event stream. Each framework adapter provides an idiomatic integration: a global callback registry for GPUI, a reactive signal for Floem, a `task` view with `MessageProxy` for Xilem, and a global signal with coroutine for Dioxus.

**Tech Stack:** Rust, `async-channel`, `url`, platform crates (`windows-registry`, `rust-ini`, `objc`), GPUI, Floem, Xilem, Dioxus.

---

## Chunk 1: Workspace & Core Crate Foundation

### Task 1.1: Create Workspace and Core Crate

**Files:**
- Create: `nexum/Cargo.toml`
- Create: `nexum/nexum-core/Cargo.toml`
- Create: `nexum/nexum-core/src/lib.rs`
- Create: `nexum/nexum-core/src/config.rs`
- Create: `nexum/nexum-core/src/error.rs`
- Create: `nexum/nexum-core/src/platform/mod.rs`

- [ ] **Step 1: Initialise workspace**

```bash
mkdir nexum
cd nexum
cargo new --lib nexum-core
```

Edit root `Cargo.toml`:

```toml
[workspace]
members = ["nexum-core"]
resolver = "2"
```

- [ ] **Step 2: Define core dependencies**

Edit `nexum-core/Cargo.toml`:

```toml
[package]
name = "nexum-core"
version = "0.1.0"
edition = "2021"

[dependencies]
url = "2.5"
async-channel = "2.3"
thiserror = "2.0"

[target.'cfg(target_os = "windows")'.dependencies]
windows-registry = "0.5"

[target.'cfg(target_os = "linux")'.dependencies]
rust-ini = "0.21"
dirs = "5.0"

[target.'cfg(target_os = "macos")'.dependencies]
objc = "0.2"
```

- [ ] **Step 3: Create module stubs**

`src/lib.rs`:

```rust
//! Nexum Core: Framework-agnostic deep linking for Rust.
//!
//! This crate handles platform-specific URL scheme registration and provides
//! an async channel for receiving incoming URLs. Framework adapters should
//! wrap `Nexum` and deliver events using the framework's idioms.

mod config;
mod error;
mod platform;

pub use config::{AppLink, Config};
pub use error::Error;

use async_channel::{Receiver, Sender};
use url::Url;

/// The main deep link manager.
pub struct Nexum {
    config: Config,
    event_tx: Sender<Vec<Url>>,
}

impl Nexum {
    /// Creates a new instance with the given configuration.
    /// On platforms that support it, this immediately checks for a current URL
    /// (e.g., from command line arguments) and sends it through the channel.
    pub fn new(config: Config) -> Self {
        let (event_tx, _) = async_channel::unbounded();
        Self { config, event_tx }
    }

    /// Registers all schemes defined in the configuration with the OS.
    pub fn register_all(&self) -> Result<(), Error> {
        #[cfg(target_os = "windows")]
        platform::windows::register_schemes(&self.config)?;
        #[cfg(target_os = "macos")]
        platform::macos::register_schemes(&self.config)?;
        #[cfg(target_os = "linux")]
        platform::linux::register_schemes(&self.config)?;
        Ok(())
    }

    /// Returns a receiver that streams incoming URLs.
    pub fn event_receiver(&self) -> Receiver<Vec<Url>> {
        self.event_tx.clone().into()
    }

    /// Returns the current deep link URLs, if any.
    /// On Windows and Linux, this checks command line arguments.
    pub fn get_current(&self) -> Option<Vec<Url>> {
        #[cfg(target_os = "windows")]
        return platform::windows::get_current_urls();
        #[cfg(target_os = "linux")]
        return platform::linux::get_current_urls();
        #[cfg(target_os = "macos")]
        return None;
    }

    /// Checks if a scheme is registered as the default handler.
    pub fn is_registered(&self, scheme: &str) -> Result<bool, Error> {
        #[cfg(target_os = "windows")]
        return platform::windows::is_registered(scheme);
        #[cfg(target_os = "linux")]
        return platform::linux::is_registered(scheme);
        #[cfg(target_os = "macos")]
        return Err(Error::UnsupportedPlatform);
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(Error::UnsupportedPlatform)
    }

    /// Unregisters a scheme (Windows/Linux only).
    pub fn unregister(&self, scheme: &str) -> Result<(), Error> {
        #[cfg(target_os = "windows")]
        platform::windows::unregister_scheme(scheme)?;
        #[cfg(target_os = "linux")]
        platform::linux::unregister_scheme(scheme)?;
        #[cfg(target_os = "macos")]
        return Err(Error::UnsupportedPlatform);
        Ok(())
    }

    /// macOS/iOS: Call this from your native app delegate when an `openURLs` event occurs.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn handle_open_urls(urls: Vec<String>) {
        platform::macos::handle_open_urls(urls);
    }
}
```

`src/config.rs`:

```rust
/// Configuration for deep link handling.
#[derive(Debug, Clone)]
pub struct Config {
    /// List of URL schemes to handle (e.g., `["myapp"]`).
    pub schemes: Vec<String>,
    /// Optional associated domains for App Links (Android/iOS).
    pub app_links: Vec<AppLink>,
}

/// An associated domain configuration for Android App Links / iOS Universal Links.
#[derive(Debug, Clone)]
pub struct AppLink {
    pub host: String,
    pub path_prefixes: Vec<String>,
}
```

`src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(target_os = "windows")]
    #[error("windows registry error: {0}")]
    Windows(#[from] windows_registry::Error),
    #[cfg(target_os = "linux")]
    #[error("ini error: {0}")]
    Ini(#[from] ini::Error),
}
```

`src/platform/mod.rs`:

```rust
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;
```

- [ ] **Step 4: Verify build**

```bash
cargo build
```

Expected: compiles without errors (unused field warnings okay).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml nexum-core
git commit -m "chore: initial workspace and core crate skeleton"
```

---

## Chunk 2: Windows Platform Implementation

### Task 2.1: Windows Registry Registration

**Files:**
- Create: `nexum-core/src/platform/windows.rs`
- Modify: `nexum-core/src/lib.rs` (add Windows-specific logic)

- [ ] **Step 1: Implement Windows registry operations**

`src/platform/windows.rs`:

```rust
use crate::{Config, Error};
use windows_registry::{CLASSES_ROOT, CURRENT_USER, LOCAL_MACHINE};
use url::Url;

/// Registers all schemes in the config with the Windows registry.
pub fn register_schemes(config: &Config) -> Result<(), Error> {
    let exe = std::env::current_exe()?;
    let exe_path = exe.to_string_lossy().to_string();

    for scheme in &config.schemes {
        let key_path = format!("Software\\Classes\\{}", scheme);
        let key = CURRENT_USER.create(&key_path)?;
        key.set_string("", format!("URL:{} protocol", scheme))?;
        key.set_string("URL Protocol", "")?;

        let icon_key = CURRENT_USER.create(format!("{}\\DefaultIcon", key_path))?;
        icon_key.set_string("", format!("\"{}\",0", exe_path))?;

        let cmd_key = CURRENT_USER.create(format!("{}\\shell\\open\\command", key_path))?;
        cmd_key.set_string("", format!("\"{}\" \"%1\"", exe_path))?;
    }

    Ok(())
}

/// Checks if a scheme is registered as the default handler.
pub fn is_registered(scheme: &str) -> Result<bool, Error> {
    let cmd_key = CLASSES_ROOT.open(format!("{}\\shell\\open\\command", scheme));
    if cmd_key.is_err() {
        return Ok(false);
    }
    let registered_cmd = cmd_key.unwrap().get_string("")?;
    let exe = std::env::current_exe()?;
    let expected = format!("\"{}\" \"%1\"", exe.to_string_lossy());
    Ok(registered_cmd == expected)
}

/// Removes a scheme's registration from the registry.
pub fn unregister_scheme(scheme: &str) -> Result<(), Error> {
    let path = format!("Software\\Classes\\{}", scheme);
    if CURRENT_USER.open(&path).is_ok() {
        CURRENT_USER.remove_tree(&path)?;
    }
    if LOCAL_MACHINE.open(&path).is_ok() {
        LOCAL_MACHINE.remove_tree(&path)?;
    }
    Ok(())
}

/// Extracts a URL from command line arguments if present.
pub fn get_current_urls() -> Option<Vec<Url>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        if let Ok(url) = Url::parse(&args[1]) {
            return Some(vec![url]);
        }
    }
    None
}
```

- [ ] **Step 2: Wire Windows into `Nexum::new` and other methods**

Modify `src/lib.rs`:

```rust
impl Nexum {
    pub fn new(config: Config) -> Self {
        let (event_tx, _) = async_channel::unbounded();
        let nexum = Self { config, event_tx };

        #[cfg(target_os = "windows")]
        if let Some(urls) = platform::windows::get_current_urls() {
            let _ = nexum.event_tx.try_send(urls);
        }
        #[cfg(target_os = "linux")]
        if let Some(urls) = platform::linux::get_current_urls() {
            let _ = nexum.event_tx.try_send(urls);
        }

        nexum
    }
}
```

- [ ] **Step 3: Build on Windows**

```bash
cargo build --target x86_64-pc-windows-msvc
```

- [ ] **Step 4: Commit**

```bash
git add nexum-core/src/platform/windows.rs nexum-core/src/lib.rs
git commit -m "feat(core): implement Windows deep link registration"
```

---

## Chunk 3: macOS Platform Implementation

### Task 3.1: macOS Info.plist Guidance and Apple Event Handling

**Files:**
- Create: `nexum-core/src/platform/macos.rs`
- Modify: `nexum-core/src/lib.rs` (add macOS hook)
- Modify: `nexum-core/README.md` (add setup instructions)

- [ ] **Step 1: Implement macOS module with Apple Event handler**

`src/platform/macos.rs`:

```rust
use crate::{Config, Error};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::OnceLock;
use url::Url;

static EVENT_TX: OnceLock<async_channel::Sender<Vec<Url>>> = OnceLock::new();

/// Stores the event sender for use by the Apple Event handler.
pub fn set_event_tx(tx: async_channel::Sender<Vec<Url>>) {
    EVENT_TX.set(tx).ok();
}

/// Registration on macOS is a no-op; users must manually edit Info.plist.
pub fn register_schemes(config: &Config) -> Result<(), Error> {
    if !config.schemes.is_empty() {
        log::warn!(
            "Deep link schemes must be manually added to Info.plist. Schemes: {:?}",
            config.schemes
        );
    }
    Ok(())
}

/// Call this from your AppDelegate's `application:openURLs:` method.
pub fn handle_open_urls(url_strings: Vec<String>) {
    let urls: Vec<Url> = url_strings
        .into_iter()
        .filter_map(|s| Url::parse(&s).ok())
        .collect();
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.try_send(urls);
    }
}
```

- [ ] **Step 2: Wire macOS into `Nexum`**

Modify `src/lib.rs`:

```rust
impl Nexum {
    pub fn new(config: Config) -> Self {
        let (event_tx, _) = async_channel::unbounded();
        #[cfg(target_os = "macos")]
        platform::macos::set_event_tx(event_tx.clone());
        // ... Windows/Linux current URL handling ...
        Self { config, event_tx }
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn handle_open_urls(urls: Vec<String>) {
        platform::macos::handle_open_urls(urls);
    }
}
```

- [ ] **Step 3: Add macOS setup documentation**

Create/modify `nexum-core/README.md`:

```markdown
## macOS Setup

Deep link registration on macOS requires adding `CFBundleURLTypes` to your app's `Info.plist`:

```xml
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>myapp</string>
        </array>
    </dict>
</array>
```

You must also forward the `openURLs` event from your `AppDelegate`:

```swift
import Cocoa

@main
class AppDelegate: NSObject, NSApplicationDelegate {
    func application(_ application: NSApplication, open urls: [URL]) {
        nexum_core::Nexum::handle_open_urls(urls.map { $0.absoluteString })
    }
}
```
```

- [ ] **Step 4: Build on macOS**

```bash
cargo build --target x86_64-apple-darwin
```

- [ ] **Step 5: Commit**

```bash
git add nexum-core/src/platform/macos.rs nexum-core/src/lib.rs nexum-core/README.md
git commit -m "feat(core): implement macOS deep link support"
```

---

## Chunk 4: Linux Platform Implementation

### Task 4.1: Linux .desktop File Registration

**Files:**
- Create: `nexum-core/src/platform/linux.rs`
- Modify: `nexum-core/src/lib.rs` (add Linux logic)
- Modify: `nexum-core/Cargo.toml` (add `dirs` dependency)

- [ ] **Step 1: Implement Linux registration using .desktop files**

`src/platform/linux.rs`:

```rust
use crate::{Config, Error};
use rust_ini::Ini;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use url::Url;

pub fn register_schemes(config: &Config) -> Result<(), Error> {
    let exe = std::env::current_exe()?;
    let bin_name = exe.file_name().unwrap().to_string_lossy();
    let desktop_file_name = format!("{}-handler.desktop", bin_name);
    let desktop_dir = dirs::data_dir()
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "data dir not found",
            ))
        })?
        .join("applications");
    fs::create_dir_all(&desktop_dir)?;
    let desktop_path = desktop_dir.join(&desktop_file_name);

    let exec = exe.to_string_lossy().to_string();
    let qualified_exec = format!("\"{}\" %u", exec);

    for scheme in &config.schemes {
        let mime_type = format!("x-scheme-handler/{}", scheme);

        if let Ok(mut desktop_file) = Ini::load_from_file(&desktop_path) {
            if let Some(section) = desktop_file.section_mut(Some("Desktop Entry")) {
                let old_mimes = section.remove("MimeType").unwrap_or_default();
                if !old_mimes.split(';').any(|m| m == mime_type) {
                    section.append("MimeType", format!("{};{}", mime_type, old_mimes));
                } else {
                    section.insert("MimeType".to_string(), old_mimes);
                }
                let old_exec = section.remove("Exec").unwrap_or_default();
                if old_exec != qualified_exec {
                    section.append("Exec", qualified_exec.clone());
                } else {
                    section.insert("Exec".to_string(), old_exec);
                }
                desktop_file.write_to_file(&desktop_path)?;
            }
        } else {
            let mut file = File::create(&desktop_path)?;
            file.write_all(
                format!(
                    "[Desktop Entry]\n\
                     Type=Application\n\
                     Name={name}\n\
                     Exec={qualified_exec}\n\
                     Terminal=false\n\
                     MimeType={mime_type}\n\
                     NoDisplay=true\n",
                    name = bin_name,
                )
                .as_bytes(),
            )?;
        }

        Command::new("update-desktop-database")
            .arg(&desktop_dir)
            .status()?;

        Command::new("xdg-mime")
            .args(["default", &desktop_file_name, &mime_type])
            .status()?;
    }

    Ok(())
}

pub fn is_registered(scheme: &str) -> Result<bool, Error> {
    let desktop_file_name = format!(
        "{}-handler.desktop",
        std::env::current_exe()?
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
    let output = Command::new("xdg-mime")
        .args([
            "query",
            "default",
            &format!("x-scheme-handler/{}", scheme),
        ])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).contains(&desktop_file_name))
}

pub fn unregister_scheme(scheme: &str) -> Result<(), Error> {
    let mimeapps_path = dirs::config_dir()
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "config dir not found",
            ))
        })?
        .join("mimeapps.list");
    let mut mimeapps = Ini::load_from_file(&mimeapps_path)?;
    let desktop_file_name = format!(
        "{}-handler.desktop",
        std::env::current_exe()?
            .file_name()
            .unwrap()
            .to_string_lossy()
    );

    if let Some(section) = mimeapps.section_mut(Some("Default Applications")) {
        let scheme_key = format!("x-scheme-handler/{}", scheme);
        if section.get(&scheme_key).unwrap_or_default() == desktop_file_name {
            section.remove(scheme_key);
        }
    }
    mimeapps.write_to_file(mimeapps_path)?;
    Ok(())
}

pub fn get_current_urls() -> Option<Vec<Url>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        if let Ok(url) = Url::parse(&args[1]) {
            return Some(vec![url]);
        }
    }
    None
}
```

- [ ] **Step 2: Wire Linux into `Nexum`**

Modify `src/lib.rs` (already partially done in Chunk 2; ensure Linux methods are called):

```rust
impl Nexum {
    pub fn new(config: Config) -> Self {
        // ... existing ...
        #[cfg(target_os = "linux")]
        if let Some(urls) = platform::linux::get_current_urls() {
            let _ = nexum.event_tx.try_send(urls);
        }
        // ...
    }

    pub fn register_all(&self) -> Result<(), Error> {
        #[cfg(target_os = "linux")]
        platform::linux::register_schemes(&self.config)?;
        // ... other platforms ...
        Ok(())
    }

    pub fn get_current(&self) -> Option<Vec<Url>> {
        #[cfg(target_os = "linux")]
        return platform::linux::get_current_urls();
        // ...
    }

    pub fn is_registered(&self, scheme: &str) -> Result<bool, Error> {
        #[cfg(target_os = "linux")]
        return platform::linux::is_registered(scheme);
        // ...
    }

    pub fn unregister(&self, scheme: &str) -> Result<(), Error> {
        #[cfg(target_os = "linux")]
        platform::linux::unregister_scheme(scheme)?;
        // ...
    }
}
```

- [ ] **Step 3: Build on Linux**

```bash
cargo build --target x86_64-unknown-linux-gnu
```

- [ ] **Step 4: Commit**

```bash
git add nexum-core/src/platform/linux.rs nexum-core/src/lib.rs
git commit -m "feat(core): implement Linux deep link registration"
```

---

## Chunk 5: GPUI Adapter Crate

### Task 5.1: Create `nexum-gpui` with Global Callback Registry

**Files:**
- Create: `nexum/nexum-gpui/Cargo.toml`
- Create: `nexum/nexum-gpui/src/lib.rs`
- Modify: `nexum/Cargo.toml` (add member)

- [ ] **Step 1: Add crate to workspace**

Edit root `Cargo.toml`:

```toml
members = ["nexum-core", "nexum-gpui"]
```

- [ ] **Step 2: Define adapter dependencies**

`nexum-gpui/Cargo.toml`:

```toml
[package]
name = "nexum-gpui"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-core = { path = "../nexum-core", version = "0.1.0" }
gpui = { workspace = true }
async-channel = "2.3"
url = "2.5"
```

- [ ] **Step 3: Implement GPUI adapter**

`src/lib.rs`:

```rust
use gpui::{App, AppContext, Global, Subscription};
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// A type alias for deep link callbacks.
pub type DeepLinkCallback = Box<dyn Fn(&[Url], &mut App) + Send + Sync>;

/// Global state holding registered callbacks.
struct DeepLinkRegistry {
    callbacks: Vec<DeepLinkCallback>,
}

impl Global for DeepLinkRegistry {}

impl DeepLinkRegistry {
    fn push(&mut self, cb: DeepLinkCallback) -> Subscription {
        self.callbacks.push(cb);
        let index = self.callbacks.len() - 1;
        // Note: In a full implementation, use a slotmap to allow removal.
        Subscription::new(move || {
            // Placeholder: would remove the callback at `index`.
        })
    }

    fn invoke(&self, urls: &[Url], cx: &mut App) {
        for cb in &self.callbacks {
            cb(urls, cx);
        }
    }
}

/// The Nexum GPUI adapter.
pub struct Nexum {
    inner: CoreNexum,
}

impl Nexum {
    /// Creates a new Nexum instance and registers all schemes.
    pub fn new(config: Config) -> Self {
        let inner = CoreNexum::new(config);
        inner.register_all().expect("Failed to register schemes");
        Self { inner }
    }

    /// Spawns a background task that listens for URLs and invokes registered callbacks.
    pub fn spawn_listener(&self, cx: &mut App) {
        if !cx.has_global::<DeepLinkRegistry>() {
            cx.set_global(DeepLinkRegistry {
                callbacks: Vec::new(),
            });
        }

        let rx = self.inner.event_receiver();
        cx.spawn(|mut cx| async move {
            while let Ok(urls) = rx.recv().await {
                cx.update(|cx| {
                    cx.update_global(|registry: &mut DeepLinkRegistry, cx| {
                        registry.invoke(&urls, cx);
                    });
                })
                .ok();
            }
        })
        .detach();
    }

    /// Registers a callback to be invoked when a deep link is received.
    /// Returns a `Subscription` that can be used to unregister.
    pub fn on_deep_link(
        cx: &mut App,
        callback: impl Fn(&[Url], &mut App) + Send + Sync + 'static,
    ) -> Subscription {
        cx.update_global(|registry: &mut DeepLinkRegistry, _| registry.push(Box::new(callback)))
    }

    /// Returns the current deep link URLs, if any.
    pub fn get_current(&self) -> Option<Vec<Url>> {
        self.inner.get_current()
    }

    /// Checks if a scheme is registered.
    pub fn is_registered(&self, scheme: &str) -> Result<bool, nexum_core::Error> {
        self.inner.is_registered(scheme)
    }

    /// Unregisters a scheme (Windows/Linux only).
    pub fn unregister(&self, scheme: &str) -> Result<(), nexum_core::Error> {
        self.inner.unregister(scheme)
    }

    /// macOS/iOS: Call this from your native app delegate.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn handle_open_urls(urls: Vec<String>) {
        CoreNexum::handle_open_urls(urls);
    }
}
```

- [ ] **Step 4: Build adapter**

```bash
cargo build -p nexum-gpui
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml nexum-gpui
git commit -m "feat(gpui): add GPUI adapter crate with global callback registry"
```

---

## Chunk 6: Floem Adapter Crate (Revised)

### Task 6.1: Create `nexum-floem` with Reactive Signal Integration

**Files:**
- Create: `nexum/nexum-floem/Cargo.toml`
- Create: `nexum/nexum-floem/src/lib.rs`
- Modify: `nexum/Cargo.toml` (add member)

- [ ] **Step 1: Add crate to workspace**

Edit root `Cargo.toml`:

```toml
members = ["nexum-core", "nexum-gpui", "nexum-floem"]
```

- [ ] **Step 2: Define dependencies**

`nexum-floem/Cargo.toml`:

```toml
[package]
name = "nexum-floem"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-core = { path = "../nexum-core", version = "0.1.0" }
floem = { workspace = true }
url = "2.5"
```

- [ ] **Step 3: Implement Floem adapter using RwSignal**

`src/lib.rs`:

```rust
use floem::reactive::RwSignal;
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// Spawns a background thread that listens for deep link URLs and updates the given `RwSignal`.
/// The signal can then be observed in a Floem view's `update` method or an effect.
pub fn spawn_deep_link_listener(config: Config, signal: RwSignal<Option<Vec<Url>>>) {
    let inner = CoreNexum::new(config);
    inner.register_all().expect("Failed to register schemes");

    std::thread::spawn(move || {
        let rx = inner.event_receiver();
        while let Ok(urls) = rx.recv_blocking() {
            signal.set(Some(urls));
        }
    });
}

/// Returns the current deep link URLs, if any.
pub fn get_current(config: &Config) -> Option<Vec<Url>> {
    CoreNexum::new(config.clone()).get_current()
}

/// Checks if a scheme is registered.
pub fn is_registered(config: &Config, scheme: &str) -> Result<bool, nexum_core::Error> {
    CoreNexum::new(config.clone()).is_registered(scheme)
}

/// Unregisters a scheme (Windows/Linux only).
pub fn unregister(config: &Config, scheme: &str) -> Result<(), nexum_core::Error> {
    CoreNexum::new(config.clone()).unregister(scheme)
}

/// macOS/iOS: Call this from your native app delegate.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn handle_open_urls(urls: Vec<String>) {
    CoreNexum::handle_open_urls(urls);
}
```

- [ ] **Step 4: Build adapter**

```bash
cargo build -p nexum-floem
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml nexum-floem
git commit -m "feat(floem): add Floem adapter using RwSignal"
```

---

## Chunk 7: Xilem Adapter Crate (Revised)

### Task 7.1: Create `nexum-xilem` with Task View and MessageProxy

**Files:**
- Create: `nexum/nexum-xilem/Cargo.toml`
- Create: `nexum/nexum-xilem/src/lib.rs`
- Modify: `nexum/Cargo.toml` (add member)

- [ ] **Step 1: Add crate to workspace**

Edit root `Cargo.toml`:

```toml
members = ["nexum-core", "nexum-gpui", "nexum-floem", "nexum-xilem"]
```

- [ ] **Step 2: Define dependencies**

`nexum-xilem/Cargo.toml`:

```toml
[package]
name = "nexum-xilem"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-core = { path = "../nexum-core", version = "0.1.0" }
xilem = { workspace = true }
url = "2.5"
```

- [ ] **Step 3: Implement Xilem adapter using `task` view**

`src/lib.rs`:

```rust
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;
use xilem::view::{task, WidgetView};

/// Creates a `task` view that spawns a deep link listener and sends messages via `MessageProxy`.
/// The `make_message` function converts a `Vec<Url>` into your application's message type.
pub fn deep_link_task<M, F>(config: Config, make_message: F) -> impl WidgetView<M>
where
    M: Send + 'static,
    F: Fn(Vec<Url>) -> M + Send + 'static,
{
    task(
        |proxy| async move {
            let inner = CoreNexum::new(config);
            inner.register_all().expect("Failed to register schemes");
            let rx = inner.event_receiver();
            while let Ok(urls) = rx.recv().await {
                let msg = make_message(urls);
                let _ = proxy.send(msg).await;
            }
        },
        |_state, _proxy| {},
    )
}

/// Returns the current deep link URLs, if any.
pub fn get_current(config: &Config) -> Option<Vec<Url>> {
    CoreNexum::new(config.clone()).get_current()
}

/// Checks if a scheme is registered.
pub fn is_registered(config: &Config, scheme: &str) -> Result<bool, nexum_core::Error> {
    CoreNexum::new(config.clone()).is_registered(scheme)
}

/// Unregisters a scheme (Windows/Linux only).
pub fn unregister(config: &Config, scheme: &str) -> Result<(), nexum_core::Error> {
    CoreNexum::new(config.clone()).unregister(scheme)
}

/// macOS/iOS: Call this from your native app delegate.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn handle_open_urls(urls: Vec<String>) {
    CoreNexum::handle_open_urls(urls);
}
```

- [ ] **Step 4: Build adapter**

```bash
cargo build -p nexum-xilem
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml nexum-xilem
git commit -m "feat(xilem): add Xilem adapter using task view and MessageProxy"
```

---

## Chunk 8: Dioxus Adapter Crate

### Task 8.1: Create `nexum-dioxus` with Global Signals and Coroutines

**Files:**
- Create: `nexum/nexum-dioxus/Cargo.toml`
- Create: `nexum/nexum-dioxus/src/lib.rs`
- Modify: `nexum/Cargo.toml` (add member)

- [ ] **Step 1: Add crate to workspace**

Edit root `Cargo.toml`:

```toml
members = ["nexum-core", "nexum-gpui", "nexum-floem", "nexum-xilem", "nexum-dioxus"]
```

- [ ] **Step 2: Define dependencies**

`nexum-dioxus/Cargo.toml`:

```toml
[package]
name = "nexum-dioxus"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-core = { path = "../nexum-core", version = "0.1.0" }
dioxus = { workspace = true }
url = "2.5"
```

- [ ] **Step 3: Implement Dioxus adapter**

`src/lib.rs`:

```rust
use dioxus::prelude::*;
use nexum_core::{Config, Nexum as CoreNexum};
use url::Url;

/// A global signal containing the most recent deep link URLs.
pub static DEEP_LINK_URLS: GlobalSignal<Option<Vec<Url>>> = Signal::global(|| None);

/// Initializes the deep link listener as a coroutine.
/// Call this from your root component.
pub fn use_deep_link_listener(config: Config) {
    let inner = CoreNexum::new(config);
    inner.register_all().expect("Failed to register schemes");

    use_coroutine(|_rx: UnboundedReceiver<()>| {
        let url_rx = inner.event_receiver();
        async move {
            while let Ok(urls) = url_rx.recv().await {
                *DEEP_LINK_URLS.write() = Some(urls);
            }
        }
    });
}

/// Alternative: returns a signal that updates with each deep link.
pub fn use_deep_link_signal(config: Config) -> Signal<Option<Vec<Url>>> {
    let signal = use_signal(|| None);
    let inner = CoreNexum::new(config);
    inner.register_all().expect("Failed to register schemes");

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let url_rx = inner.event_receiver();
        async move {
            while let Ok(urls) = url_rx.recv().await {
                signal.set(Some(urls));
            }
        }
    });

    signal
}

/// Returns the current deep link URLs, if any.
pub fn get_current(config: &Config) -> Option<Vec<Url>> {
    CoreNexum::new(config.clone()).get_current()
}

/// Checks if a scheme is registered.
pub fn is_registered(config: &Config, scheme: &str) -> Result<bool, nexum_core::Error> {
    CoreNexum::new(config.clone()).is_registered(scheme)
}

/// Unregisters a scheme (Windows/Linux only).
pub fn unregister(config: &Config, scheme: &str) -> Result<(), nexum_core::Error> {
    CoreNexum::new(config.clone()).unregister(scheme)
}

/// macOS/iOS: Call this from your native app delegate.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn handle_open_urls(urls: Vec<String>) {
    CoreNexum::handle_open_urls(urls);
}
```

- [ ] **Step 4: Build adapter**

```bash
cargo build -p nexum-dioxus
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml nexum-dioxus
git commit -m "feat(dioxus): add Dioxus adapter crate using global signals and coroutines"
```

---

## Chunk 9: Example Applications

### Task 9.1: GPUI Example

**Files:**
- Create: `nexum/examples/gpui-basic/Cargo.toml`
- Create: `nexum/examples/gpui-basic/src/main.rs`

- [ ] **Step 1: Set up GPUI example**

`examples/gpui-basic/Cargo.toml`:

```toml
[package]
name = "gpui-basic"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-gpui = { path = "../../nexum-gpui" }
gpui = { workspace = true }
```

`src/main.rs`:

```rust
use gpui::*;
use nexum_gpui::Nexum;

struct HelloWorld;

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(gpui::white())
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(gpui::black())
            .child("Hello, deep links!")
    }
}

fn main() {
    let config = nexum_core::Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };

    let nexum = Nexum::new(config);

    Application::new().run(move |cx: &mut App| {
        nexum.spawn_listener(cx);

        Nexum::on_deep_link(cx, |urls, _cx| {
            println!("Deep link received: {:?}", urls);
        });

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| HelloWorld)
        })
        .unwrap();
    });
}
```

- [ ] **Step 2: Run GPUI example**

```bash
cd examples/gpui-basic
cargo run
```

- [ ] **Step 3: Commit**

```bash
git add examples/gpui-basic
git commit -m "docs: add GPUI example application"
```

### Task 9.2: Floem Example (Revised)

**Files:**
- Create: `nexum/examples/floem-basic/Cargo.toml`
- Create: `nexum/examples/floem-basic/src/main.rs`

- [ ] **Step 1: Set up Floem example**

`examples/floem-basic/Cargo.toml`:

```toml
[package]
name = "floem-basic"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-floem = { path = "../../nexum-floem" }
floem = { workspace = true }
```

`src/main.rs`:

```rust
use floem::{
    reactive::{create_effect, create_rw_signal, RwSignal},
    views::{label, v_stack, Decorators},
    View,
};
use nexum_floem::spawn_deep_link_listener;
use url::Url;

fn app_view() -> impl View {
    let deep_link_urls: RwSignal<Option<Vec<Url>>> = create_rw_signal(None);
    let config = nexum_core::Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };
    spawn_deep_link_listener(config, deep_link_urls);

    // Create a label that updates when the signal changes.
    let display_text = create_rw_signal("No deep link received".to_string());
    create_effect(move |_| {
        if let Some(urls) = deep_link_urls.get() {
            display_text.set(format!("Last URL: {:?}", urls));
        }
    });

    v_stack((
        label(|| "Hello, deep links!".to_string()),
        label(move || display_text.get()),
    ))
    .style(|s| s.padding(20.0))
}

fn main() {
    floem::launch(app_view);
}
```

- [ ] **Step 2: Run Floem example**

```bash
cd examples/floem-basic
cargo run
```

- [ ] **Step 3: Commit**

```bash
git add examples/floem-basic
git commit -m "docs: add Floem example application"
```

### Task 9.3: Xilem Example (Revised)

**Files:**
- Create: `nexum/examples/xilem-basic/Cargo.toml`
- Create: `nexum/examples/xilem-basic/src/main.rs`

- [ ] **Step 1: Set up Xilem example**

`examples/xilem-basic/Cargo.toml`:

```toml
[package]
name = "xilem-basic"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-xilem = { path = "../../nexum-xilem" }
xilem = { workspace = true }
url = "2.5"
```

`src/main.rs`:

```rust
use xilem::{
    view::{button, flex, label},
    App, AppLauncher, WidgetView,
};
use url::Url;

#[derive(Debug)]
enum AppMessage {
    DeepLink(Vec<Url>),
    Clear,
}

struct AppState {
    last_urls: Option<Vec<Url>>,
}

impl AppState {
    fn new() -> Self {
        Self { last_urls: None }
    }
}

fn app_logic(state: &mut AppState) -> impl WidgetView<AppMessage> {
    let config = nexum_core::Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };

    flex((
        label("Hello, deep links!".to_string()),
        label(match &state.last_urls {
            Some(urls) => format!("Last URL: {:?}", urls),
            None => "No deep link received".to_string(),
        }),
        button("Clear", |_| AppMessage::Clear),
        // The deep link task is inserted as a view.
        nexum_xilem::deep_link_task(config, AppMessage::DeepLink),
    ))
}

impl App for AppState {
    type Message = AppMessage;

    fn update(&mut self, message: Self::Message) {
        match message {
            AppMessage::DeepLink(urls) => self.last_urls = Some(urls),
            AppMessage::Clear => self.last_urls = None,
        }
    }

    fn view(&mut self) -> xilem::WidgetView<Self::Message> {
        app_logic(self)
    }
}

fn main() {
    AppLauncher::new(AppState::new()).run();
}
```

- [ ] **Step 2: Run Xilem example**

```bash
cd examples/xilem-basic
cargo run
```

- [ ] **Step 3: Commit**

```bash
git add examples/xilem-basic
git commit -m "docs: add Xilem example application"
```

### Task 9.4: Dioxus Example

**Files:**
- Create: `nexum/examples/dioxus-basic/Cargo.toml`
- Create: `nexum/examples/dioxus-basic/src/main.rs`

- [ ] **Step 1: Set up Dioxus example**

`examples/dioxus-basic/Cargo.toml`:

```toml
[package]
name = "dioxus-basic"
version = "0.1.0"
edition = "2021"

[dependencies]
nexum-dioxus = { path = "../../nexum-dioxus" }
dioxus = { workspace = true, features = ["desktop"] }
```

`src/main.rs`:

```rust
use dioxus::prelude::*;
use nexum_dioxus::DEEP_LINK_URLS;

fn App() -> Element {
    let config = nexum_core::Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };
    nexum_dioxus::use_deep_link_listener(config);

    let urls = DEEP_LINK_URLS.read();

    rsx! {
        div {
            h1 { "Hello, deep links!" }
            p {
                match urls.as_ref() {
                    Some(urls) => format!("Last URL: {:?}", urls),
                    None => "No deep link received".to_string(),
                }
            }
        }
    }
}

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(dioxus::desktop::Config::new().with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Nexum Dioxus Example")
        ))
        .launch(App);
}
```

- [ ] **Step 2: Run Dioxus example**

```bash
cd examples/dioxus-basic
cargo run
```

- [ ] **Step 3: Commit**

```bash
git add examples/dioxus-basic
git commit -m "docs: add Dioxus example application"
```

---

## Chunk 10: Documentation and Final Polish

### Task 10.1: Write Comprehensive README Files

**Files:**
- Create/Modify: `nexum/README.md`
- Modify: `nexum-core/README.md`
- Create: `nexum-gpui/README.md`
- Create: `nexum-floem/README.md`
- Create: `nexum-xilem/README.md`
- Create: `nexum-dioxus/README.md`

- [ ] **Step 1: Workspace README**

`nexum/README.md`:

```markdown
# Nexum

Framework‑agnostic deep linking for Rust.

Nexum provides a unified API for registering custom URL schemes and receiving deep link events across Windows, macOS, and Linux. It includes a core crate (`nexum-core`) with all platform logic, plus adapter crates for popular Rust UI frameworks.

## Supported Platforms

| Platform | Registration | URL Detection |
|----------|--------------|---------------|
| Windows  | Registry     | CLI arguments |
| macOS    | Info.plist   | Apple Events  |
| Linux    | .desktop     | CLI arguments |

## Framework Adapters

| Adapter         | Integration Mechanism                         | Status      |
|-----------------|-----------------------------------------------|-------------|
| `nexum-gpui`    | `Global` trait + callback registry            | ✅ Ready    |
| `nexum-floem`   | `RwSignal`                                    | ✅ Ready    |
| `nexum-xilem`   | `task` view + `MessageProxy`                  | ✅ Ready    |
| `nexum-dioxus`  | `GlobalSignal` + `use_coroutine`              | ✅ Ready    |

## Quick Start

Choose the adapter for your framework. Example for GPUI:

```toml
[dependencies]
nexum-gpui = "0.1.0"
```

See the [examples](examples/) directory for complete demos.

## Platform Setup

- **Windows**: Automatic via `register_all()`.
- **macOS**: Manually add `CFBundleURLTypes` to `Info.plist` and forward `openURLs` events.
- **Linux**: Automatic via `.desktop` file creation.

## License

MIT OR Apache-2.0
```

- [ ] **Step 2: Core README**

Add detailed platform setup instructions and API reference to `nexum-core/README.md`.

- [ ] **Step 3: Adapter READMEs**

For each adapter crate, write a short README with:
- Installation
- Basic usage example
- Link to the example application

- [ ] **Step 4: Commit**

```bash
git add README.md nexum-core/README.md nexum-*/README.md
git commit -m "docs: add comprehensive README files for all crates"
```

### Task 10.2: Final Verification

- [ ] Run `cargo build --workspace` to ensure all crates compile.
- [ ] Run `cargo test --workspace` (if any tests).
- [ ] Test each example manually on its target platform.

---

## Final Check

- [ ] All crates build on Windows, macOS, and Linux.
- [ ] All examples run correctly and handle deep links.
- [ ] Documentation is complete.

**Plan complete and saved to `docs/superpowers/plans/2026-04-17-nexum-deep-linking-complete-revised.md`. Ready to execute?**
