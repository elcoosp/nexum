# nexum-gpui

GPUI adapter for nexum deep linking. GPUI surfaces incoming URLs through
its own `on_open_urls` application callback; this adapter bridges that
callback into a nexum `DeepLinkHub` so the rest of your code can stay
GPUI-agnostic.

## Usage

```rust,ignore
use gpui::Application;
use nexum_core::Config;
use nexum_gpui::setup_deep_links;

let app = Application::headless();
let config = Config::builder()
    .scheme("myapp")
    .build()
    .expect("valid config");
let handle = setup_deep_links(&app, &config).expect("deep links failed");

app.run(move |cx| {
    let view = cx.new(|_| ());
    nexum_gpui::attach_deep_link(
        handle,
        view,
        cx,
        |_view: &mut (), _cx: &mut gpui::Context<()>, url| {
            println!("opened {url}");
        },
    );
});
```

## API

| Item | Description |
| :--- | :--- |
| `setup_deep_links(&Application, &Config)` | Registers the schemes, installs the global hub and forwards GPUI's `on_open_urls` into it. Returns the consumer `DeepLinkHandle`. |
| `attach_deep_link(handle, view, cx, on_url)` | Drives `on_url` on the given entity for every deep link received on `handle`. |

## Platform notes

- **macOS** — scheme declarations must be present in your bundle's
  `Info.plist`; nexum then receives `GURL` Apple events.
- **Windows / Linux** — schemes are registered per-user (registry /
  `.desktop` + `xdg-mime`), and cold-start URLs arrive as command-line
  events.

> [!NOTE]
> This crate depends on GPUI through the Zed git registry and is therefore
> **not part of the default workspace build**; build it standalone with
> `cargo build -p nexum-gpui` when the Zed dependency is acceptable.

