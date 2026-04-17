// platform/macos.rs
use objc2::define_class;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{msg_send, sel, ClassType};
use objc2_foundation::{NSAppleEventDescriptor, NSAppleEventManager, NSString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use url::Url;

use crate::{Config, Error};

static EVENT_TX: OnceLock<async_channel::Sender<Vec<Url>>> = OnceLock::new();
static HANDLER_REGISTERED: AtomicBool = AtomicBool::new(false);
static HANDLER_INSTANCE: OnceLock<Retained<NexumURLHandler>> = OnceLock::new();

pub fn set_event_tx(tx: async_channel::Sender<Vec<Url>>) {
    EVENT_TX.set(tx).ok();
}

pub fn register_schemes(config: &Config) -> Result<(), Error> {
    if !config.schemes.is_empty() {
        eprintln!(
            "Nexum: Deep link schemes must be manually added to Info.plist. Schemes: {:?}",
            config.schemes
        );
    }
    Ok(())
}

pub fn handle_open_urls(url_strings: Vec<String>) {
    let urls: Vec<Url> = url_strings
        .into_iter()
        .filter_map(|s| Url::parse(&s).ok())
        .collect();
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.try_send(urls);
    }
}

pub fn get_current_urls() -> Option<Vec<Url>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        if let Ok(url) = Url::parse(&args[1]) {
            return Some(vec![url]);
        }
    }
    None
}

// -----------------------------------------------------------------------------
// Objective‑C class that receives the Apple Event
// -----------------------------------------------------------------------------
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "NexumURLHandler"]
    struct NexumURLHandler;

    impl NexumURLHandler {
        #[unsafe(method(handleGetURLEvent:withReplyEvent:))]
        fn handle_get_url_event(&self, event: &NSAppleEventDescriptor, _reply: &NSAppleEventDescriptor) {
            // kAEKeyDirectObject = '----'
            let direct_object_key = u32::from_be_bytes(*b"----");
            // Use msg_send to call paramDescriptorForKeyword:
            let param: Option<Retained<NSAppleEventDescriptor>> = unsafe {
                msg_send![event, paramDescriptorForKeyword: direct_object_key]
            };
            if let Some(param) = param {
                // Get string value from the descriptor
                let url_string: Option<Retained<NSString>> = unsafe { msg_send![&param, stringValue] };
                if let Some(url_string) = url_string {
                    let url_str = url_string.to_string();
                    eprintln!("Nexum: Apple Event URL: {}", url_str);
                    if let Some(tx) = EVENT_TX.get() {
                        if let Ok(url) = Url::parse(&url_str) {
                            let _ = tx.try_send(vec![url]);
                        }
                    }
                }
            }
        }
    }
);

unsafe impl Send for NexumURLHandler {}
unsafe impl Sync for NexumURLHandler {}

// -----------------------------------------------------------------------------
// Public registration function
// -----------------------------------------------------------------------------
pub fn setup_apple_event_listener() {
    if HANDLER_REGISTERED.swap(true, Ordering::Relaxed) {
        return;
    }

    // Create instance using NSObject's +new class method
    let handler: Retained<NexumURLHandler> = unsafe { msg_send![NexumURLHandler::class(), new] };
    HANDLER_INSTANCE.set(handler.clone()).ok();

    let manager = NSAppleEventManager::sharedAppleEventManager();

    // kInternetEventClass = 'GURL'
    let event_class = u32::from_be_bytes(*b"GURL");
    // kAEGetURL = 'GURL'
    let event_id = u32::from_be_bytes(*b"GURL");

    unsafe {
        let _: () = msg_send![
            &*manager,
            setEventHandler: &*handler,
            andSelector: sel!(handleGetURLEvent:withReplyEvent:),
            forEventClass: event_class,
            andEventID: event_id
        ];
    }

    eprintln!("Nexum: Registered Apple Event handler for kAEGetURL.");
}
