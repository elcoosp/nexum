use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(target_os = "windows")]
    #[error("windows registry error: {0}")]
    Windows(#[from] windows_registry::Error),
}
