//! Linux registration: a `Desktop Entry` file plus `xdg-mime` wiring.
//!
//! The cross-platform pieces that are worth testing live here as pure
//! helpers (`desktop_entry`, `quoted_exec`, `sanitized_app_name`); only the
//! final file writes and the `xdg-mime` invocation touch the system.

use std::path::{Path, PathBuf};
use std::process::Command;

use nexum_core::{Config, Error, Result};

pub(crate) fn register(config: &Config) -> Result<()> {
    let exe = std::env::current_exe()?;
    write_registration(dirs::data_local_dir(), &exe, config)
}

/// Writes the `.desktop` entry and wires every scheme with `xdg-mime`.
///
/// Split from [`register`] so the whole flow is testable with explicit
/// inputs: `data_dir` is passed as an [`Option`] so the "no XDG data
/// directory" error path can be exercised, and `exe` is arbitrary so the
/// `nexum-app` fallback for unrepresentable names is reachable.
pub(crate) fn write_registration(
    data_dir: Option<PathBuf>,
    exe: &Path,
    config: &Config,
) -> Result<()> {
    let data_dir = data_dir
        .ok_or_else(|| Error::Platform("cannot determine the XDG data directory".to_owned()))?;
    let app_name = match sanitized_app_name(exe) {
        empty if empty.is_empty() => "nexum-app".to_owned(),
        app_name => app_name,
    };

    let applications_dir = data_dir.join("applications");
    std::fs::create_dir_all(&applications_dir)?;

    let entry = desktop_entry(&app_name, exe, config);
    std::fs::write(applications_dir.join(format!("{app_name}.desktop")), entry)?;

    for scheme in &config.schemes {
        let mime_type = format!("x-scheme-handler/{scheme}");
        let status = Command::new("xdg-mime")
            .args(["default", &format!("{app_name}.desktop"), &mime_type])
            .status()?;
        if !status.success() {
            return Err(Error::Platform(format!(
                "`xdg-mime default` for {mime_type} failed with {status}"
            )));
        }
    }
    Ok(())
}

/// Renders the content of the application's desktop entry.
///
/// The `MimeType` list uses the desktop-entry convention of `;`-separated
/// values with a trailing `;`, `Exec` is quoted/escaped by [`quoted_exec`],
/// and `Icon` falls back to the executable path when
/// [`Config::icon_path`](nexum_core::Config) is unset.
#[must_use]
pub(crate) fn desktop_entry(app_name: &str, exe: &Path, config: &Config) -> String {
    let mime_types = config
        .schemes
        .iter()
        .map(|scheme| format!("x-scheme-handler/{scheme}"))
        .collect::<Vec<_>>()
        .join(";");
    let icon = config.icon_path.as_deref().unwrap_or(exe).display();
    format!(
        "[Desktop Entry]\nType=Application\nName={app_name}\nExec={} %u\nStartupNotify=false\nIcon={icon}\nMimeType={mime_types};\n",
        quoted_exec(exe)
    )
}

/// Formats an executable path for the desktop-entry `Exec` key.
///
/// Paths containing spaces or quotes are wrapped in double quotes with
/// embedded quotes escaped, and `%` is doubled (desktop entries treat `%`
/// as a field-code prefix).
#[must_use]
pub(crate) fn quoted_exec(exe: &Path) -> String {
    let path = exe.to_string_lossy().replace('%', "%%");
    if path.contains([' ', '"']) {
        format!("\"{}\"", path.replace('"', "\\\""))
    } else {
        path
    }
}

/// Derives the desktop-file name from the executable, keeping only
/// characters valid in a desktop-file id (`[A-Za-z0-9._-]`).
#[must_use]
pub(crate) fn sanitized_app_name(exe: &Path) -> String {
    exe.file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn config(schemes: &[&str], icon: Option<&str>) -> Config {
        let mut config = Config::new(schemes.iter().copied());
        if let Some(icon) = icon {
            config = config.with_icon_path(icon);
        }
        config
    }

    #[test]
    fn desktop_entry_without_icon_falls_back_to_exe() {
        let entry = desktop_entry(
            "myapp",
            Path::new("/opt/apps/myapp"),
            &config(&["foo", "bar"], None),
        );
        assert_eq!(
            entry,
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=myapp\n\
             Exec=/opt/apps/myapp %u\n\
             StartupNotify=false\n\
             Icon=/opt/apps/myapp\n\
             MimeType=x-scheme-handler/foo;x-scheme-handler/bar;\n"
        );
    }

    #[test]
    fn desktop_entry_with_icon() {
        let entry = desktop_entry(
            "myapp",
            Path::new("/opt/apps/myapp"),
            &config(&["foo"], Some("/icons/foo.png")),
        );
        assert!(entry.contains("Icon=/icons/foo.png\n"));
    }

    #[test]
    fn quoted_exec_handles_plain_paths() {
        assert_eq!(quoted_exec(Path::new("/usr/bin/app")), "/usr/bin/app");
    }

    #[test]
    fn quoted_exec_quotes_spaces() {
        assert_eq!(
            quoted_exec(Path::new("/opt/My Apps/app")),
            "\"/opt/My Apps/app\""
        );
    }

    #[test]
    fn quoted_exec_escapes_quotes_and_percent() {
        assert_eq!(quoted_exec(Path::new("/op%t/a\"pp")), "\"/op%%t/a\\\"pp\"");
    }

    #[test]
    fn sanitized_app_name_keeps_valid_characters_only() {
        assert_eq!(
            sanitized_app_name(&PathBuf::from("/opt/my-app_v2.bin")),
            "my-app_v2"
        );
        assert_eq!(sanitized_app_name(&PathBuf::from("wéird/app!")), "app");
        assert_eq!(sanitized_app_name(Path::new("")), "");
    }
}
