//! Nexum Leptos deep-link demo (`examples/leptos-basic`).
//!
//! The app renders in the browser (wasm32, client-side rendering via Trunk).
//! In the browser the operating system cannot register URL schemes or
//! deliver open events, so the demo owns the `DeepLinkHub` and plays the
//! role `nexum_platform` plays on desktop: UI controls and a `?url=`
//! cold-start query parameter push URLs into the hub, and the
//! `nexum_leptos::signal_from_handle` bridge forwards them into a
//! fine-grained signal that drives the whole UI.
//!
//! On a desktop target the only difference is the producer side: call
//! `nexum_leptos::setup(&config)` once, pass the returned handle to
//! `signal_from_handle` and the same pipeline delivers real OS deep links.

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        demo::start();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("`leptos-basic` renders in the browser (wasm32, CSR).");
        println!("Build and serve it with Trunk:  cargo install trunk && trunk serve");
    }
}

/// The browser demo. Compiled for `wasm32` only — the DOM, the event
/// handlers and the cold-start scan of `window.location` only exist there;
/// on native the binary prints build instructions instead.
#[cfg(target_arch = "wasm32")]
mod demo {
    use std::sync::Arc;

    use leptos::prelude::*;
    use nexum_leptos::{DeepLinkHub, signal_from_handle};

    /// Scheme the demo listens for.
    pub const SCHEME: &str = "nexumdemo";

    /// Starts as `None` and holds the latest deep-link URL afterwards.
    type LatestUrl = RwSignal<Option<String>>;

    /// Mounts the demo into the document body.
    pub fn start() {
        mount_to_body(app);
    }

    fn app() -> impl IntoView {
        // Producer side. In the browser the OS cannot deliver URL events, so
        // the demo owns the hub and plays the role of `nexum_platform` on
        // desktop.
        let hub = Arc::new(DeepLinkHub::new());

        // A `?url=` query parameter acts like a cold-start command-line URL.
        if let Some(url) = cold_start_url() {
            hub.push_launch_url(url);
        }

        // Consumer side: the nexum-leptos bridge (the handle's only consumer).
        let latest: LatestUrl = signal_from_handle(&hub.handle());

        // History of received deep links, maintained by a fine-grained effect.
        let log = RwSignal::new(Vec::<String>::new());
        let received = Memo::new(move |_| log.with(Vec::len));
        Effect::new(move |_| {
            if let Some(url) = latest.get() {
                log.update(|entries| entries.push(url));
            }
        });

        // Simulation controls: a free-text field plus a few preset links.
        let draft = RwSignal::new(format!("{SCHEME}://open/settings"));

        let open_draft = Arc::clone(&hub);
        let open_preset = Arc::clone(&hub);
        let presets: [(&str, &str); 3] = [
            ("open/settings", "Settings"),
            ("open/doc/42", "Doc 42"),
            ("download/report.pdf", "Report"),
        ];

        view! {
            <main class="shell">
                <header>
                    <h1>"Nexum " <span class="times">"×"</span> " Leptos"</h1>
                    <p class="subtitle">
                        "Deep links bridged into fine-grained signals"
                    </p>
                </header>

                <section class="card highlight">
                    <h2>"Latest deep link"</h2>
                    <p class="url" class:empty=move || latest.get().is_none()>
                        {move || {
                            latest
                                .get()
                                .unwrap_or_else(|| format!("Waiting for a {SCHEME}:// deep link…"))
                        }}
                    </p>
                    <p class="count">
                        {move || received.get()}
                        " link(s) received"
                    </p>
                </section>

                <section class="card">
                    <h2>"Simulate an incoming deep link"</h2>
                    <div class="row">
                        <input
                            type="text"
                            prop:value=move || draft.get()
                            on:input=move |event| draft.set(event_target_value(&event))
                        />
                        <button class="primary" on:click=move |_| {
                            open_draft.push_url(draft.get_untracked());
                        }>"Open"</button>
                    </div>
                    <div class="row presets">
                        {presets
                            .into_iter()
                            .map(|(path, label)| {
                                let push = Arc::clone(&open_preset);
                                view! {
                                    <button on:click=move |_| {
                                        push.push_url(format!("{SCHEME}://{path}"));
                                    }>{label}</button>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                    <p class="hint">
                        "Pushing into the " <code>"DeepLinkHub"</code>
                        " simulates what the OS layer delivers after "
                        <code>"nexum_leptos::setup"</code> " on desktop."
                    </p>
                </section>

                <section class="card">
                    <h2>"Received links"</h2>
                    <ul class="log">
                        {move || {
                            log.get()
                                .into_iter()
                                .rev()
                                .map(|entry| {
                                    view! { <li><code>{entry}</code></li> }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </ul>
                </section>

                <footer>
                    <p>
                        "Browser demo — scheme registration is simulated by design. "
                        "Desktop builds register " {SCHEME}
                        ":// through " <code>"nexum_leptos::setup"</code> " instead."
                    </p>
                </footer>
            </main>
        }
    }

    /// Reads the `url` query parameter, mirroring the command-line cold-start
    /// scan that `nexum_platform::register` performs on Windows and Linux.
    fn cold_start_url() -> Option<String> {
        let search = window().location().search().ok()?;
        let query = search.strip_prefix('?')?;
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=')
                && key == "url"
            {
                return Some(value.to_owned());
            }
        }
        None
    }
}
