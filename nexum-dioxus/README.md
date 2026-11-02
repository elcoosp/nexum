# nexum-dioxus

Dioxus desktop adapter for nexum deep linking.

- `setup` — registers the schemes and stores the consumer handle.
- `with_deep_link_handler` — wires tao's `Event::Opened` into the hub.
- `launch_desktop` — the one-call path: setup + handler + launch.
- `use_deep_link` — exposes the latest URL as a `Signal<Option<String>>`
  inside components.

## Usage

```rust,ignore
use dioxus::prelude::*;
use dioxus_desktop::{Config as DesktopConfig, LogicalSize, WindowBuilder};
use nexum_dioxus::{launch_desktop, use_deep_link, Config};

fn app() -> Element {
    let deep_link = use_deep_link();
    rsx! {
        div {
            h1 { "Hello, deep links!" }
            p { "Last deep link: {deep_link:?}" }
        }
    }
}

fn main() {
    let nexum_config = Config::builder().scheme("myapp").build().expect("valid config");
    let desktop_config = DesktopConfig::new().with_window(
        WindowBuilder::new()
            .with_title("Deep Link Demo")
            .with_inner_size(LogicalSize::new(800.0, 600.0)),
    );
    launch_desktop(&nexum_config, desktop_config, app);
}
```

Prefer manual control? Call `setup(&config)` once, then either

- pass your `dioxus_desktop::Config` through
  `with_deep_link_handler(desktop_config)` before `launch`, or
- consume `nexum_dioxus::handle()` directly (blocking / async / stream).

## Cold start

On Windows and Linux, URLs passed on the command line when the process is
launched are delivered as `Source::CommandLine` events automatically during
`setup`, so a cold start from an OS shortcut behaves exactly like a runtime
open.

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
