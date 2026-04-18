use nexum_core::{Config, DeepLinkHandle, DeepLinkHub};
use nexum_platform;

/// Initializes the deep link system and returns a blocking receiver.
pub fn setup(config: Config) -> DeepLinkHandle {
    let hub = DeepLinkHub::new();
    let handle = hub.handle();
    nexum_platform::set_hub(hub);
    nexum_platform::register(&config);
    handle
}

#[cfg(feature = "signal")]
pub fn signal(config: Config) -> floem::reactive::ReadSignal<Option<String>> {
    use crossbeam_channel::bounded;
    use floem::ext_event::create_signal_from_channel;

    let handle = setup(config);
    let (tx, rx) = bounded::<String>(1);

    std::thread::spawn(move || {
        while let Ok(url) = handle.recv_blocking() {
            let _ = tx.send(url);
        }
    });

    create_signal_from_channel(rx)
}
