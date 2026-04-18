use async_channel::{unbounded, Sender};
use nexum_core::{Config, DeepLinkHandle};
use once_cell::sync::OnceCell;
use std::fs::OpenOptions;
use std::io::Write;

static URL_SENDER: OnceCell<Sender<String>> = OnceCell::new();

macro_rules! log {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        eprintln!("{}", msg);
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/nexum-floem.log")
        {
            let _ = writeln!(f, "{}", msg);
        }
    }};
}

fn push_url(url: String) {
    log!("[nexum-floem] push_url called with: {}", url);
    if let Some(tx) = URL_SENDER.get() {
        match tx.try_send(url) {
            Ok(()) => log!("[nexum-floem] URL sent to channel"),
            Err(e) => log!("[nexum-floem] Failed to send URL: {:?}", e),
        }
    } else {
        log!("[nexum-floem] URL_SENDER not set!");
    }
}

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::{define_class, msg_send, sel, MainThreadOnly};
#[cfg(target_os = "macos")]
use objc2_foundation::{
    MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSObject, NSObjectProtocol,
    NSString,
};

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

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
                log!("[nexum-floem] handleAppleEvent called");
                let param: Option<Retained<NSAppleEventDescriptor>> = unsafe {
                    msg_send![event, paramDescriptorForKeyword: 0x2d2d2d2d_u32]
                };

                if let Some(param) = param {
                    let url_str: Option<Retained<NSString>> =
                        unsafe { msg_send![&param, stringValue] };

                    if let Some(url_str) = url_str {
                        let s = url_str.to_string();
                        log!("[nexum-floem] Extracted URL: {}", s);
                        push_url(s);
                    } else {
                        log!("[nexum-floem] No string value in param");
                    }
                } else {
                    log!("[nexum-floem] No param descriptor for keyword");
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
        log!("[nexum-floem] register_delegate called");
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
        log!("[nexum-floem] Apple Event handler registered");

        std::mem::forget(handler);
    }
}

#[cfg(target_os = "macos")]
pub use macos::register_delegate;

#[cfg(not(target_os = "macos"))]
pub fn register_delegate() {}

#[cfg(target_os = "windows")]
fn register_windows(config: &Config) {
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
fn register_linux(config: &Config) {
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

/// Initializes the deep link system and registers the OS-level handler.
pub fn setup(_config: Config) -> DeepLinkHandle {
    log!(
        "[nexum-floem] setup called on thread: {:?}",
        std::thread::current().id()
    );
    // Create channel and store sender
    let (tx, rx) = unbounded();
    URL_SENDER.set(tx).expect("nexum_floem::setup called twice");
    log!("[nexum-floem] Channel created, sender stored");

    // Platform registration
    #[cfg(target_os = "windows")]
    register_windows(&_config);
    #[cfg(target_os = "linux")]
    register_linux(&_config);
    #[cfg(target_os = "macos")]
    register_delegate(); // User must also configure Info.plist with CFBundleURLTypes

    // Process initial command‑line URL (Windows/Linux)
    #[cfg(not(target_os = "macos"))]
    if let Some(url) = std::env::args().nth(1) {
        push_url(url);
    }

    DeepLinkHandle::new(rx)
}

#[cfg(feature = "signal")]
pub fn create_deep_link_signal(config: Config) -> floem::reactive::RwSignal<Option<String>> {
    use floem::reactive::{create_rw_signal, SignalUpdate};
    let signal = create_rw_signal(None);
    let handle = setup(config);
    std::thread::spawn(move || {
        while let Ok(url) = handle.recv_blocking() {
            signal.set(Some(url));
        }
    });
    signal
}
