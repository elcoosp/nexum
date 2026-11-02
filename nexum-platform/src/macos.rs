//! macOS registration: Apple-event handling for `GURL` events.
//!
//! The OS delivers deep links as Apple events of class `'GURL'` / id
//! `'GURL'`. Scheme *declarations* come from the bundle's `Info.plist`
//! (`CFBundleURLTypes`); this module only installs the runtime handler.

use nexum_core::{Config, Error, Result};
use objc2::rc::Retained;
use objc2::{MainThreadOnly, define_class, msg_send, sel};
use objc2_foundation::{
    MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSObject, NSObjectProtocol,
    NSString,
};

use super::push_launch_url;

pub(crate) fn register(_config: &Config) -> Result<()> {
    register_delegate()
}

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
                    // The Apple event always carries a cold-start style URL
                    // from the OS point of view; the running app sees it as
                    // a system hand-off, so the regular source is used.
                    let _ = push_launch_url(url_str.to_string());
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

/// Installs the `GURL`/`GURL` Apple-event handler on the main thread.
///
/// # Errors
///
/// Returns [`Error::Platform`] when not called from the main thread.
///
/// # Panics
///
/// Panics if the Objective-C runtime fails to register the handler, which
/// indicates a broken Objective-C runtime rather than a recoverable error.
pub fn register_delegate() -> Result<()> {
    let mtm = MainThreadMarker::new().ok_or_else(|| {
        Error::Platform("deep links must be registered on the main thread".into())
    })?;
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

    // The handler must outlive the event manager; leak it deliberately.
    std::mem::forget(handler);
    Ok(())
}
