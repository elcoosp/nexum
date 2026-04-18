use crossbeam_channel::bounded;
use dioxus::prelude::*;
use dioxus_signals::GlobalSignal;
use nexum_core::{Config, DeepLinkHandle, DeepLinkHub};

static DEEP_LINK: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Initializes deep linking and returns a handle to listen for URLs.
pub fn init(config: Config) -> DeepLinkHandle {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();
    nexum_platform::set_hub(hub);
    nexum_platform::register(&config);
    handle
}

/// A hook that provides a reactive signal updated with received deep link URLs.
pub fn use_deep_link(handle: DeepLinkHandle) -> Signal<Option<String>> {
    let mut signal = use_signal(|| None);

    use_coroutine(|_rx: UnboundedReceiver<()>| async move {
        let (tx, rx) = bounded::<String>(1);
        std::thread::spawn(move || {
            while let Ok(url) = handle.recv_blocking() {
                let _ = tx.send(url);
            }
        });

        while let Ok(url) = rx.recv() {
            signal.set(Some(url));
        }
    });

    signal
}

/// Convenience: initializes and returns a signal directly.
pub fn init_signal(config: Config) -> Signal<Option<String>> {
    let handle = init(config);
    use_deep_link(handle)
}
