use nexum_core::Config;

fn main() {
    let config = Config {
        schemes: vec!["myapp".to_string()],
        app_links: vec![],
    };

    let rx = nexum_xilem::create_deep_link_listener(config);

    println!("Xilem deep link listener started.");

    while let Ok(urls) = rx.recv_blocking() {
        println!("Received deep link: {:?}", urls);
    }
}
