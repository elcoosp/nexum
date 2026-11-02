# nexum-platform

OS-level URL-scheme registration for nexum. This crate owns the
process-global [`DeepLinkHub`](#usage) and performs the platform-specific
registration of URL schemes:

| Platform | Mechanism | Cold-start URL |
| :--- | :--- | :--- |
| Windows | Per-user registry (`Software\Classes`) | command-line args |
| macOS | Apple events (`GURL`) | Apple event |
| Linux | `.desktop` entry + `xdg-mime` | command-line args |

Applications normally call one helper — `init` — which creates the global
hub, registers the schemes and returns the consumer handle:

```rust,ignore
use nexum_core::Config;
use nexum_platform::init;

let config = Config::builder().scheme("myapp").build().expect("valid config");
let handle = init(&config).expect("registration failed");

// Blocking or async access to incoming deep links:
if let Ok(url) = handle.recv_blocking() {
    println!("opened {url}");
}
```

## What it does per platform

- **Linux** — writes `<app>.desktop` into `XDG_DATA_HOME/applications` and
  wires every configured scheme with `xdg-mime default`.
- **Windows** — creates `Software\Classes\<scheme>` registry keys
  (protocol marker, `DefaultIcon`, `shell\open\command`) via
  `windows-registry`.
- **macOS** — installs the Apple-event handler for `GURL` events using
  `objc2`; the `Info.plist` scheme declarations must be provided by your
  app bundle.

On Windows and Linux the command line is scanned for cold-start URLs and
each match is delivered with `Source::CommandLine`.

## Beyond registration

- **`single_instance`** (feature `single-instance`) — a second launch
  forwards its deep-link URLs to the running instance over a local socket
  (Unix domain socket on unix targets, loopback TCP on Windows) and exits.
  ```rust,ignore
  use nexum_platform::single_instance::{self, Acquisition};

  match single_instance::acquire(&config, &cold_start_urls)? {
      Acquisition::Primary(_guard) => run_app(),  // keep the guard alive
      Acquisition::HandedOff { forwarded } => println!("forwarded {forwarded}; exiting"),
  }
  ```
- **`plist`** — generate the `CFBundleURLTypes` declaration for an
  `Info.plist` (`url_types_xml`) and validate an existing one
  (`missing_schemes`). On macOS, `validate_bundle(&config)` checks the
  running bundle directly.
- **`mobile`** — Android `<intent-filter>` generator
  (`android_manifest_intent_filter`), iOS `Info.plist` generator (shared
  `plist::url_types_xml`) and the runtime intake entry points
  `push_intent_url` (Android) / `push_open_urls` (iOS) for framework glue.
- **Bounded hubs** — `init_with(&config, nexum_core::DeepLinkHub::bounded(16))`
  caps the event queue; `try_push_url` reports `Error::QueueFull` instead
  of blocking.

## Testing note

Platform registration mutates process-global state. All global-state tests
in this crate deliberately live in a single serial `#[test]` function; see
[`docs/COVERAGE.md`](../docs/COVERAGE.md) for the full strategy.

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
