/// Configuration for deep link handling.
#[derive(Debug, Clone)]
pub struct Config {
    /// List of URL schemes to handle (e.g., `["myapp"]`).
    pub schemes: Vec<String>,
    /// Optional associated domains for App Links (Android/iOS).
    pub app_links: Vec<AppLink>,
}

/// An associated domain configuration for Android App Links / iOS Universal Links.
#[derive(Debug, Clone)]
pub struct AppLink {
    pub host: String,
    pub path_prefixes: Vec<String>,
}
