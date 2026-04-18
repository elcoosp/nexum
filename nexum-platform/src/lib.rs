use nexum_core::{Config, DeepLinkHub};
use once_cell::sync::OnceCell;

static HUB: OnceCell<DeepLinkHub> = OnceCell::new();

/// Sets the global deep link hub. Must be called before any platform registration.
pub fn set_hub(hub: DeepLinkHub) {
    HUB.set(hub).unwrap();
}

/// Pushes a URL to the global hub. Called by platform event handlers.
pub fn push_url(url: String) {
    if let Some(hub) = HUB.get() {
        hub.push_url(url);
    }
}

#[cfg(target_os = "windows")]
pub fn register_windows(config: &Config) {
    use windows_registry::*;
    for scheme in &config.schemes {
        let scheme_key = format!("Software\\Classes\\{}", scheme);
        let key = CURRENT_USER.create(&scheme_key).unwrap();
        key.set_string("", &format!("URL:{} Protocol", scheme))
            .unwrap();
        key.set_string("URL Protocol", "").unwrap();

        let icon_key = CURRENT_USER
            .create(&format!("{}\\DefaultIcon", scheme_key))
            .unwrap();
        let exe_path = std::env::current_exe().unwrap();
        icon_key
            .set_string("", &format!("{},0", exe_path.to_string_lossy()))
            .unwrap();

        let cmd_key = CURRENT_USER
            .create(&format!("{}\\shell\\open\\command", scheme_key))
            .unwrap();
        cmd_key
            .set_string("", &format!("\"{}\" \"%1\"", exe_path.to_string_lossy()))
            .unwrap();
    }
}

#[cfg(target_os = "linux")]
pub fn register_linux(config: &Config) {
    use std::fs;
    use std::process::Command;

    let exe_path = std::env::current_exe().unwrap();
    let exe_str = exe_path.to_string_lossy();
    let app_name = std::env::args()
        .next()
        .unwrap_or_else(|| "nexum-app".to_string());
    let app_name = std::path::Path::new(&app_name)
        .file_stem()
        .unwrap()
        .to_string_lossy();

    let desktop_entry = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Exec={} %u\n\
         StartupNotify=false\n\
         MimeType={}\n",
        app_name,
        exe_str,
        config
            .schemes
            .iter()
            .map(|s| format!("x-scheme-handler/{}", s))
            .collect::<Vec<_>>()
            .join(";")
    );

    let desktop_dir = dirs::data_local_dir().unwrap().join("applications");
    fs::create_dir_all(&desktop_dir).unwrap();
    let desktop_path = desktop_dir.join(format!("{}.desktop", app_name));
    fs::write(&desktop_path, desktop_entry).unwrap();

    for scheme in &config.schemes {
        let mime_type = format!("x-scheme-handler/{}", scheme);
        Command::new("xdg-mime")
            .args(["default", &format!("{}.desktop", app_name), &mime_type])
            .status()
            .unwrap();
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::push_url;
    use objc2::rc::Retained;
    use objc2::{define_class, msg_send, sel, MainThreadOnly};
    use objc2_foundation::{
        MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSObject, NSObjectProtocol,
        NSString,
    };

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "NexumURLHandler"]
        struct URLHandler;

        unsafe impl NSObjectProtocol for URLHandler {}

        impl URLHandler {
            #[unsafe(method(handleAppleEvent:withReplyEvent:))]
            fn handle_apple_event(
                &self,
                event: &NSAppleEventDescriptor,
                _reply: &NSAppleEventDescriptor,
            ) {
                let param: Option<Retained<NSAppleEventDescriptor>> = unsafe {
                    msg_send![event, paramDescriptorForKeyword: 0x2d2d2d2d_u32]
                };

                if let Some(param) = param {
                    let url_str: Option<Retained<NSString>> =
                        unsafe { msg_send![&param, stringValue] };

                    if let Some(url_str) = url_str {
                        push_url(url_str.to_string());
                    }
                }
            }
        }
    );

    impl URLHandler {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(());
            unsafe { msg_send![super(this), init] }
        }
    }

    pub fn register_delegate() {
        let mtm = MainThreadMarker::new().expect("must be called on the main thread");
        let handler = URLHandler::new(mtm);
        let manager = NSAppleEventManager::sharedAppleEventManager();

        unsafe {
            let _: () = msg_send![
                &manager,
                setEventHandler: &*handler,
                andSelector: sel!(handleAppleEvent:withReplyEvent:),
                forEventClass: 0x4755524c_u32, // 'GURL'
                andEventID: 0x4755524c_u32      // 'GURL'
            ];
        }

        std::mem::forget(handler);
    }
}

#[cfg(target_os = "macos")]
pub use macos::register_delegate;

#[cfg(not(target_os = "macos"))]
pub fn register_delegate() {}

/// Performs all platform‑specific registration for the given config.
/// Must be called after `set_hub`.
pub fn register(_config: &Config) {
    #[cfg(target_os = "windows")]
    register_windows(_config);
    #[cfg(target_os = "linux")]
    register_linux(_config);
    #[cfg(target_os = "macos")]
    register_delegate();

    // Process initial command‑line URL (Windows/Linux)
    #[cfg(not(target_os = "macos"))]
    if let Some(url) = std::env::args().nth(1) {
        push_url(url);
    }
}
