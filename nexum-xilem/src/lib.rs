use async_channel::{unbounded, Sender};
use nexum_core::{Config, DeepLinkHandle};
use once_cell::sync::OnceCell;

static URL_SENDER: OnceCell<Sender<String>> = OnceCell::new();
static WAKE_FN: OnceCell<Box<dyn Fn() + Send + Sync>> = OnceCell::new();

fn push_url(url: String) {
    if let Some(tx) = URL_SENDER.get() {
        let _ = tx.try_send(url);
    }
    // Tell the Xilem event loop to wake up and rebuild the view
    if let Some(wake_fn) = WAKE_FN.get() {
        wake_fn();
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

/// Initializes the deep link system and registers the OS-level handler.
pub fn setup(_config: Config) -> DeepLinkHandle {
    let (tx, rx) = unbounded();
    URL_SENDER.set(tx).expect("nexum_xilem::setup called twice");
    register_delegate();
    DeepLinkHandle::new(rx)
}

/// Wires up the Xilem proxy so incoming deep links trigger a UI rebuild.
pub fn set_wake_fn(f: impl Fn() + Send + Sync + 'static) {
    if WAKE_FN.set(Box::new(f)).is_err() {
        panic!("wake fn already set");
    }
}
