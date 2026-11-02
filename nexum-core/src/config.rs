use std::path::PathBuf;

use crate::error::{Error, Result};

/// An app link (a.k.a. universal link) mapping: an `https://` host plus the
/// path prefixes this application claims.
///
/// App links are declared alongside URL schemes in [`Config`]; platform
/// integrations use them when the OS needs a list of web origins the app
/// handles (e.g. macOS Big Sur style universal links).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct AppLink {
    /// The host this application claims, e.g. `example.com`.
    pub host: String,
    /// Path prefixes under `host` that should be routed to the app, each
    /// starting with `/`. An empty list claims the whole host.
    pub path_prefixes: Vec<String>,
}

impl AppLink {
    /// Creates an app link from a host and an iterator of path prefixes.
    ///
    /// ```
    /// use nexum_core::AppLink;
    ///
    /// let link = AppLink::new("example.com", ["/app", "/invite"]);
    /// assert_eq!(link.host, "example.com");
    /// assert_eq!(link.path_prefixes, ["/app", "/invite"]);
    /// ```
    #[must_use]
    pub fn new(
        host: impl Into<String>,
        path_prefixes: impl IntoIterator<Item: Into<String>>,
    ) -> Self {
        Self {
            host: host.into(),
            path_prefixes: path_prefixes.into_iter().map(Into::into).collect(),
        }
    }
}

/// The deep-link configuration of an application.
///
/// Schemes are validated by [`Config::validate`] (or automatically by
/// [`ConfigBuilder::build`]) so that invalid schemes fail loudly at startup
/// instead of producing mysterious operating-system failures later.
///
/// # Example
///
/// ```
/// use nexum_core::{AppLink, Config};
///
/// let config = Config::new(["myapp"])
///     .with_app_link(AppLink::new("example.com", ["/app"]));
/// config.validate().expect("valid config");
/// assert_eq!(config.schemes, ["myapp"]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Config {
    /// URL schemes this application handles, e.g. `myapp`.
    pub schemes: Vec<String>,
    /// App links (universal links) this application claims.
    pub app_links: Vec<AppLink>,
    /// Optional icon shown by the OS for the protocol handler. Used for the
    /// Windows `DefaultIcon` registry value and the Linux `Icon=` entry.
    pub icon_path: Option<PathBuf>,
}

impl Config {
    /// Creates a configuration from a list of URL schemes.
    ///
    /// Call [`Config::validate`] to check the schemes against the RFC 3986
    /// grammar, or use [`Config::builder`] for a validated, chainable API.
    #[must_use]
    pub fn new(schemes: impl IntoIterator<Item: Into<String>>) -> Self {
        Self {
            schemes: schemes.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Starts building a [`Config`] with a validated, chainable API.
    ///
    /// ```
    /// use nexum_core::Config;
    ///
    /// let config = Config::builder().scheme("myapp").build().expect("valid");
    /// ```
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Sets the app links of the configuration, consuming and returning it.
    #[must_use]
    pub fn with_app_links(mut self, links: impl IntoIterator<Item = AppLink>) -> Self {
        self.app_links = links.into_iter().collect();
        self
    }

    /// Appends one app link to the configuration, consuming and returning it.
    #[must_use]
    pub fn with_app_link(mut self, link: AppLink) -> Self {
        self.app_links.push(link);
        self
    }

    /// Sets the icon path of the configuration, consuming and returning it.
    #[must_use]
    pub fn with_icon_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoSchemes`] when no scheme is configured,
    /// [`Error::InvalidScheme`] when a scheme does not match the RFC 3986
    /// `scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )` grammar, and
    /// [`Error::InvalidHost`] when an app link has an empty host.
    pub fn validate(&self) -> Result<()> {
        if self.schemes.is_empty() {
            return Err(Error::NoSchemes);
        }
        for scheme in &self.schemes {
            Self::validate_scheme(scheme)?;
        }
        for link in &self.app_links {
            if link.host.trim().is_empty() {
                return Err(Error::InvalidHost(link.host.clone()));
            }
        }
        Ok(())
    }

    /// Checks one scheme against the RFC 3986 scheme grammar.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidScheme`] if the scheme is not valid.
    pub fn validate_scheme(scheme: &str) -> Result<()> {
        let mut chars = scheme.chars();
        let valid = matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
        if valid {
            Ok(())
        } else {
            Err(Error::InvalidScheme(scheme.to_owned()))
        }
    }

    /// Extracts command-line arguments that look like a deep link handled by
    /// this configuration.
    ///
    /// On Windows and Linux a cold-start URL arrives as a regular command
    /// line argument. This helper picks up **every** argument that parses as
    /// an absolute URL whose scheme is one of the configured schemes, and
    /// ignores everything else (application flags, files, garbage).
    ///
    /// Scheme comparison is case-insensitive; URLs are returned in argument
    /// order, unmodified.
    pub fn extract_launch_urls<I>(&self, args: I) -> Vec<String>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let schemes: Vec<String> = self
            .schemes
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        args.into_iter()
            .filter_map(|arg| {
                let arg = arg.as_ref();
                let parsed = url::Url::parse(arg).ok()?;
                schemes
                    .iter()
                    .any(|scheme| *scheme == parsed.scheme())
                    .then(|| arg.to_owned())
            })
            .collect()
    }
}

/// Builder for [`Config`] that validates on [`ConfigBuilder::build`].
///
/// ```
/// use nexum_core::{AppLink, Config};
///
/// let config = Config::builder()
///     .scheme("myapp")
///     .schemes(["also-myapp"])
///     .app_link(AppLink::new("example.com", ["/app"]))
///     .icon_path("/usr/share/icons/myapp.png")
///     .build()
///     .expect("valid config");
/// assert_eq!(config.schemes, ["myapp", "also-myapp"]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    schemes: Vec<String>,
    app_links: Vec<AppLink>,
    icon_path: Option<PathBuf>,
}

impl ConfigBuilder {
    /// Appends one URL scheme.
    #[must_use]
    pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
        self.schemes.push(scheme.into());
        self
    }

    /// Appends several URL schemes.
    #[must_use]
    pub fn schemes(mut self, schemes: impl IntoIterator<Item: Into<String>>) -> Self {
        self.schemes.extend(schemes.into_iter().map(Into::into));
        self
    }

    /// Appends one app link.
    #[must_use]
    pub fn app_link(mut self, link: AppLink) -> Self {
        self.app_links.push(link);
        self
    }

    /// Appends several app links.
    #[must_use]
    pub fn app_links(mut self, links: impl IntoIterator<Item = AppLink>) -> Self {
        self.app_links.extend(links);
        self
    }

    /// Sets the icon path shown by the OS for the protocol handler.
    #[must_use]
    pub fn icon_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    /// Builds and validates the configuration.
    ///
    /// # Errors
    ///
    /// Same as [`Config::validate`].
    pub fn build(self) -> Result<Config> {
        let config = Config {
            schemes: self.schemes,
            app_links: self.app_links,
            icon_path: self.icon_path,
        };
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_link_new_collects_prefixes() {
        let link = AppLink::new("example.com", ["/a", "/b"]);
        assert_eq!(link.host, "example.com");
        assert_eq!(link.path_prefixes, vec!["/a", "/b"]);
    }

    #[test]
    fn config_new_maps_schemes() {
        let config = Config::new(["a", "b"]);
        assert_eq!(config.schemes, vec!["a", "b"]);
        assert_eq!(config.app_links.len(), 0);
        assert!(config.icon_path.is_none());
    }

    #[test]
    fn config_default_is_empty() {
        let config = Config::default();
        assert_eq!(config.schemes.len(), 0);
    }

    #[test]
    fn config_with_methods_chain() {
        // `with_app_links` replaces the list, `with_app_link` appends one.
        let config = Config::new(["a"])
            .with_app_links([
                AppLink::new("h1", [] as [&str; 0]),
                AppLink::new("h2", ["/p"]),
            ])
            .with_app_link(AppLink::new("h3", [] as [&str; 0]))
            .with_icon_path("/icons/app.png");
        assert_eq!(config.app_links.len(), 3);
        assert_eq!(
            config.icon_path.as_deref(),
            Some(std::path::Path::new("/icons/app.png"))
        );
    }

    #[test]
    fn validate_accepts_valid_schemes() {
        let config = Config::new(["myapp", "my-app+2.x"]);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_scheme_list() {
        let config = Config::default();
        assert!(matches!(config.validate(), Err(Error::NoSchemes)));
    }

    #[test]
    fn validate_rejects_bad_schemes() {
        for scheme in ["", "1abc", "sp ace", "schème", "a:b"] {
            let config = Config::new([scheme]);
            assert!(
                matches!(config.validate(), Err(Error::InvalidScheme(_))),
                "{scheme:?}"
            );
        }
    }

    #[test]
    fn validate_rejects_empty_app_link_host() {
        let config = Config::new(["a"]).with_app_link(AppLink::new("  ", [] as [&str; 0]));
        assert!(matches!(config.validate(), Err(Error::InvalidHost(_))));
    }

    #[test]
    fn validate_scheme_accepts_single_alpha() {
        assert!(Config::validate_scheme("a").is_ok());
    }

    #[test]
    fn builder_builds_and_validates() {
        let config = Config::builder()
            .scheme("myapp")
            .schemes(["b", "c"])
            .app_link(AppLink::new("example.com", ["/app"]))
            .app_links([AppLink::new("other.org", [] as [&str; 0])])
            .icon_path("/i.png")
            .build()
            .expect("valid");
        assert_eq!(config.schemes, vec!["myapp", "b", "c"]);
        assert_eq!(config.app_links.len(), 2);
        assert_eq!(
            config.icon_path.as_deref(),
            Some(std::path::Path::new("/i.png"))
        );
    }

    #[test]
    fn builder_surfaces_validation_errors() {
        let error = Config::builder().scheme("9bad").build().unwrap_err();
        assert!(matches!(error, Error::InvalidScheme(ref s) if s == "9bad"));
    }

    #[test]
    fn extract_launch_urls_filters_by_scheme() {
        let config = Config::new(["myapp"]);
        let urls = config.extract_launch_urls([
            "myapp://open/1",
            "--flag",
            "https://example.com",
            "MYAPP://open/2",
            "otherapp://nope",
            "not a url",
        ]);
        assert_eq!(urls, vec!["myapp://open/1", "MYAPP://open/2"]);
    }

    #[test]
    fn extract_launch_urls_empty_and_case_insensitive_config() {
        let config = Config::new(["MyApp"]);
        assert_eq!(config.extract_launch_urls([] as [&str; 0]).len(), 0);
        assert_eq!(config.extract_launch_urls(["myapp://x"]), vec!["myapp://x"]);
    }

    #[test]
    fn extract_launch_urls_accepts_scheme_only_urls() {
        let config = Config::new(["myapp"]);
        assert_eq!(config.extract_launch_urls(["myapp:"]), vec!["myapp:"]);
    }
}
