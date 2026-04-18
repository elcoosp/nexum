use crossbeam_channel::bounded;
use floem::ext_event::create_signal_from_channel;
use floem::prelude::*;
use floem::reactive::ReadSignal;
use floem::views::{label, v_stack};
use floem::View;
use nexum_core::Config;
use nexum_floem::setup;

fn app_view(url_signal: ReadSignal<Option<String>>) -> impl View {
    v_stack((
        label(|| "Hello, deep links!".to_string()),
        label(move || {
            url_signal
                .get()
                .unwrap_or_else(|| "No deep link received".to_string())
        }),
    ))
}

fn main() {
    // Create a crossbeam channel (capacity 1 is fine)
    let (tx, rx) = bounded::<String>(1);

    // Set up deep link handling in background thread
    let config = Config {
        schemes: vec!["floem".to_string()],
        app_links: vec![],
    };
    let handle = setup(config);
    std::thread::spawn(move || {
        eprintln!("[bg-thread] Receiver thread started");
        while let Ok(url) = handle.recv_blocking() {
            eprintln!("[bg-thread] Received URL: {}", url);
            // Send to crossbeam channel – this wakes the UI thread
            let _ = tx.send(url);
        }
    });

    // Create a reactive signal from the channel
    let url_signal = create_signal_from_channel(rx);

    floem::launch(move || app_view(url_signal));
}
