use crate::{Config, Error};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use url::Url;

/// Registers all schemes by creating/updating a .desktop file and running
/// xdg-mime to associate each scheme with the application.
pub fn register_schemes(config: &Config) -> Result<(), Error> {
    let exe = std::env::current_exe()?;
    let bin_name = exe
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let desktop_file_name = format!("{}-handler.desktop", bin_name);
    let desktop_dir = dirs::data_dir()
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "XDG data directory not found",
            ))
        })?
        .join("applications");

    fs::create_dir_all(&desktop_dir)?;
    let desktop_path = desktop_dir.join(&desktop_file_name);

    let exec = format!("\"{}\" %u", exe.to_string_lossy());
    let mime_types: Vec<String> = config
        .schemes
        .iter()
        .map(|s| format!("x-scheme-handler/{}", s))
        .collect();
    let mime_str = mime_types.join(";") + ";";

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={name}\n\
         Exec={exec}\n\
         Terminal=false\n\
         MimeType={mimes}\n\
         NoDisplay=true\n",
        name = bin_name,
        exec = exec,
        mimes = mime_str,
    );

    fs::write(&desktop_path, content)?;

    // Update the desktop database so the DE picks up the new file.
    let _ = Command::new("update-desktop-database")
        .arg(&desktop_dir)
        .status();

    // Register each scheme as the default handler.
    for scheme in &config.schemes {
        let mime_type = format!("x-scheme-handler/{}", scheme);
        let _ = Command::new("xdg-mime")
            .args(["default", &desktop_file_name, &mime_type])
            .status();
    }

    Ok(())
}

/// Checks whether this application is the default handler for the given scheme.
pub fn is_registered(scheme: &str) -> Result<bool, Error> {
    let desktop_file_name = format!(
        "{}-handler.desktop",
        std::env::current_exe()?
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    let output = Command::new("xdg-mime")
        .args([
            "query",
            "default",
            &format!("x-scheme-handler/{}", scheme),
        ])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).contains(&desktop_file_name))
}

/// Removes the default-handler association for a scheme from mimeapps.list.
pub fn unregister_scheme(scheme: &str) -> Result<(), Error> {
    let mimeapps_path = dirs::config_dir()
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "XDG config directory not found",
            ))
        })?
        .join("mimeapps.list");

    if !mimeapps_path.exists() {
        return Ok(());
    }

    let desktop_file_name = format!(
        "{}-handler.desktop",
        std::env::current_exe()?
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );

    let content = fs::read_to_string(&mimeapps_path)?;
    let scheme_key = format!("x-scheme-handler/{}", scheme);
    let new_content: String = content
        .lines()
        .filter(|line| !(line.contains(&scheme_key) && line.contains(&desktop_file_name)))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&mimeapps_path, new_content)?;
    Ok(())
}

/// Extracts a URL from command line arguments if present.
pub fn get_current_urls() -> Option<Vec<Url>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        if let Ok(url) = Url::parse(&args[1]) {
            return Some(vec![url]);
        }
    }
    None
}
