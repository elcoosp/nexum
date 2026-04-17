use crate::{Config, Error};
use url::Url;
use windows_registry::{CLASSES_ROOT, CURRENT_USER, LOCAL_MACHINE};

/// Registers all schemes in the config with the Windows registry.
pub fn register_schemes(config: &Config) -> Result<(), Error> {
    let exe = std::env::current_exe()?;
    let exe_path = exe.to_string_lossy().to_string();

    for scheme in &config.schemes {
        let key_path = format!("Software\\Classes\\{}", scheme);
        let key = CURRENT_USER.create(&key_path)?;
        key.set_string("", format!("URL:{} protocol", scheme))?;
        key.set_string("URL Protocol", "")?;

        let icon_key = CURRENT_USER.create(format!("{}\\DefaultIcon", key_path))?;
        icon_key.set_string("", format!("\"{}\",0", exe_path))?;

        let cmd_key = CURRENT_USER.create(format!("{}\\shell\\open\\command", key_path))?;
        cmd_key.set_string("", format!("\"{}\" \"%1\"", exe_path))?;
    }

    Ok(())
}

/// Checks if a scheme is registered as the default handler.
pub fn is_registered(scheme: &str) -> Result<bool, Error> {
    let cmd_key = CLASSES_ROOT.open(format!("{}\\shell\\open\\command", scheme));
    if cmd_key.is_err() {
        return Ok(false);
    }
    let registered_cmd = cmd_key.unwrap().get_string("")?;
    let exe = std::env::current_exe()?;
    let expected = format!("\"{}\" \"%1\"", exe.to_string_lossy());
    Ok(registered_cmd == expected)
}

/// Removes a scheme's registration from the registry.
pub fn unregister_scheme(scheme: &str) -> Result<(), Error> {
    let path = format!("Software\\Classes\\{}", scheme);
    if CURRENT_USER.open(&path).is_ok() {
        CURRENT_USER.remove_tree(&path)?;
    }
    if LOCAL_MACHINE.open(&path).is_ok() {
        LOCAL_MACHINE.remove_tree(&path)?;
    }
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
