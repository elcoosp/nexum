use floem::prelude::*;
use floem::reactive::ReadSignal;
use nexum_floem::Config;
use nexum_floem::signal;

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
    let config = Config::builder()
        .scheme("floem")
        .build()
        .expect("valid config");

    let url_signal = signal(&config);

    floem::launch(move || app_view(url_signal));
}
