/// The single error type of the nexum stack.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The configuration does not declare any URL scheme.
    #[error("at least one URL scheme must be configured")]
    NoSchemes,

    /// A configured scheme does not match the RFC 3986 scheme grammar.
    #[error(
        "`{0}` is not a valid URL scheme (RFC 3986: ALPHA *( ALPHA / DIGIT / \"+\" / \"-\" / \".\" ))"
    )]
    InvalidScheme(String),

    /// An app link was declared with an empty host.
    #[error("`{0}` is not a valid app-link host")]
    InvalidHost(String),

    /// The global deep-link hub was already initialized for this process.
    #[error("the deep-link hub is already initialized for this process")]
    HubAlreadySet,

    /// A platform operation ran before `init`/`set_hub` was called.
    #[error("no deep-link hub is initialized yet; call `nexum_platform::init` first")]
    HubNotSet,

    /// Every sender and receiver of the hub channel has been dropped.
    #[error("the deep-link channel has been shut down")]
    HubClosed,

    /// A deep-link URL could not be parsed as a [`url::Url`].
    #[error("`{0}` is not a valid URL: {1}")]
    MalformedUrl(String, #[source] url::ParseError),

    /// A platform backend reported a failure that has no more precise
    /// representation.
    #[error("platform registration failed: {0}")]
    Platform(String),

    /// An underlying I/O operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Convenience alias used throughout the nexum crates.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_are_human_readable() {
        assert_eq!(
            Error::NoSchemes.to_string(),
            "at least one URL scheme must be configured"
        );
        assert_eq!(
            Error::InvalidScheme("9bad".into()).to_string(),
            "`9bad` is not a valid URL scheme (RFC 3986: ALPHA *( ALPHA / DIGIT / \"+\" / \"-\" / \".\" ))"
        );
        assert_eq!(
            Error::InvalidHost(" ".into()).to_string(),
            "` ` is not a valid app-link host"
        );
        assert_eq!(
            Error::HubAlreadySet.to_string(),
            "the deep-link hub is already initialized for this process"
        );
        assert_eq!(
            Error::HubNotSet.to_string(),
            "no deep-link hub is initialized yet; call `nexum_platform::init` first"
        );
        assert_eq!(
            Error::HubClosed.to_string(),
            "the deep-link channel has been shut down"
        );
        assert_eq!(
            Error::Platform("boom".into()).to_string(),
            "platform registration failed: boom"
        );
        assert_eq!(
            Error::MalformedUrl("nope".into(), url::Url::parse("nope").unwrap_err()).to_string(),
            "`nope` is not a valid URL: relative URL without a base"
        );
        let io_error = std::io::Error::other("io boom");
        assert_eq!(Error::from(io_error).to_string(), "io boom");
    }

    #[test]
    fn error_is_debug_and_send_sync() {
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<Error>();
        assert!(format!("{:?}", Error::HubClosed).contains("HubClosed"));
    }
}
