//! Windows registration: per-user registry keys for every configured scheme.
//!
//! For each scheme this creates `Software\Classes\<scheme>` with:
//!
//! * the `URL:<scheme> Protocol` default value and the empty `URL Protocol`
//!   marker (what makes Windows treat the key as a protocol handler),
//! * a `DefaultIcon` value (`Config::icon_path` or the running executable),
//! * a `shell\open\command` that re-launches the executable with `"%1"`.

use std::path::Path;

use nexum_core::{Config, Error, Result};

pub(crate) fn register(config: &Config) -> Result<()> {
    use windows_registry::*;

    let exe = std::env::current_exe()?;
    let exe_path = exe.to_string_lossy();
    let icon = config
        .icon_path
        .as_ref()
        .map(|path| format!("{},0", path.display()))
        .unwrap_or_else(|| format!("{exe_path},0"));

    for scheme in &config.schemes {
        let key = CURRENT_USER
            .create(format!("Software\\Classes\\{scheme}"))
            .map_err(|error| Error::Platform(error.to_string()))?;
        key.set_string("", &format!("URL:{scheme} Protocol"))
            .map_err(|error| Error::Platform(error.to_string()))?;
        key.set_string("URL Protocol", "")
            .map_err(|error| Error::Platform(error.to_string()))?;

        let icon_key = CURRENT_USER
            .create(format!("Software\\Classes\\{scheme}\\DefaultIcon"))
            .map_err(|error| Error::Platform(error.to_string()))?;
        icon_key
            .set_string("", &icon)
            .map_err(|error| Error::Platform(error.to_string()))?;

        let command_key = CURRENT_USER
            .create(format!("Software\\Classes\\{scheme}\\shell\\open\\command"))
            .map_err(|error| Error::Platform(error.to_string()))?;
        command_key
            .set_string("", &command_line(&exe))
            .map_err(|error| Error::Platform(error.to_string()))?;
    }
    Ok(())
}

/// Builds the `shell\open\command` value for the running executable.
///
/// The path is double-quoted so paths with spaces survive the round trip
/// through the registry; `%1` is the placeholder Windows substitutes the
/// deep-link URL into.
#[must_use]
pub(crate) fn command_line(exe: &Path) -> String {
    format!("\"{}\" \"%1\"", exe.display())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::command_line;

    #[test]
    fn command_line_quotes_the_executable() {
        assert_eq!(
            command_line(Path::new("C:\\Program Files\\My App\\app.exe")),
            "\"C:\\Program Files\\My App\\app.exe\" \"%1\""
        );
    }
}
