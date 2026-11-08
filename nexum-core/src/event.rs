/// How a deep-link URL reached the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Source {
    /// The URL was delivered by the operating system while the application
    /// was already running (registry launch hand-off on Windows, an Apple
    /// event on macOS, a `DBus`/`xdg-open` hand-off on Linux).
    #[default]
    System,
    /// The URL was found on the command line of the current process, i.e.
    /// the application was cold-started to open the link.
    CommandLine,
}

/// A deep-link URL plus metadata about how it arrived.
///
/// URLs are transported as raw strings and parsed lazily with
/// [`DeepLinkEvent::parse_url`], so a malformed URL never blocks the push
/// path (e.g. the UI thread) and can be reported in the consumer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeepLinkEvent {
    /// The raw URL, e.g. `myapp://open/settings`.
    pub url: String,
    /// How the URL reached the application.
    pub source: Source,
}

impl DeepLinkEvent {
    /// Creates an event from a raw URL and its source.
    #[must_use]
    pub fn new(url: impl Into<String>, source: Source) -> Self {
        Self {
            url: url.into(),
            source,
        }
    }

    /// Parses the raw URL into a [`url::Url`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::MalformedUrl`](crate::Error::MalformedUrl) when the
    /// payload is not a valid absolute URL.
    pub fn parse_url(&self) -> crate::Result<url::Url> {
        url::Url::parse(&self.url)
            .map_err(|error| crate::Error::MalformedUrl(self.url.clone(), error))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use super::*;

    #[test]
    fn source_default_is_system() {
        assert_eq!(Source::default(), Source::System);
    }

    #[test]
    fn event_new_sets_fields() {
        let event = DeepLinkEvent::new("myapp://x", Source::CommandLine);
        assert_eq!(event.url, "myapp://x");
        assert_eq!(event.source, Source::CommandLine);
    }

    #[test]
    fn event_parse_url_ok() {
        let event = DeepLinkEvent::new("myapp://open/1", Source::System);
        let url = event.parse_url().expect("valid url");
        assert_eq!(url.scheme(), "myapp");
        // In `scheme://host/path`, the segment after `//` is the host.
        assert_eq!(url.host_str(), Some("open"));
        assert_eq!(url.path(), "/1");
    }

    #[test]
    fn event_parse_url_error_carries_raw_url() {
        let event = DeepLinkEvent::new("not a url", Source::System);
        let error = event.parse_url().expect_err("invalid");
        assert!(matches!(error, Error::MalformedUrl(raw, _) if raw == "not a url"));
    }

    #[test]
    fn event_is_cloneable_and_comparable() {
        let a = DeepLinkEvent::new("myapp://x", Source::System);
        let b = a.clone();
        assert_eq!(a, b);
        assert!(format!("{a:?}").contains("myapp://x"));
    }
}
