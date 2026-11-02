# nexum-xilem

Xilem adapter for nexum deep linking. Call `setup` once at startup, then
wrap any view with `with_deep_links`; the callback runs on the app thread
for every deep link.

## Usage

```rust,ignore
use nexum_core::Config;
use nexum_xilem::{setup, with_deep_links};
use xilem::view::{flex_col, label};
use xilem::WidgetView;

struct AppState {
    last_url: Option<String>,
}

fn app(state: &mut AppState) -> impl WidgetView<AppState> {
    let last = state.last_url.clone().unwrap_or_else(|| "waiting…".into());
    flex_col((
        label(last),
        with_deep_links(
            flex_col(()),
            |state: &mut AppState, url: String| {
                state.last_url = Some(url);
            },
        ),
    ))
}

fn main() -> xilem::Result<()> {
    let config = Config::builder().scheme("myapp").build().expect("valid config");
    setup(&config)?; // registers schemes before the event loop starts
    xilem::run(app, AppState { last_url: None })
}
```

## API

| Item | Description |
| :--- | :--- |
| `setup(&Config)` | Registers the schemes and installs the global hub. |
| `handle()` | Access the global `DeepLinkHandle` for manual consumption. |
| `with_deep_links(callback, view)` | Wraps a view; the callback fires on the app thread per deep link. |

Internally the adapter polls the hub with `task_raw` + a `MessageProxy`
fork so deep links wake the Xilem event loop without blocking the UI
thread.

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
