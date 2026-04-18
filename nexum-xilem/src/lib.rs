use async_channel::{unbounded, Receiver, Sender};
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::{define_class, msg_send, sel, MainThreadOnly};
#[cfg(target_os = "macos")]
use objc2_foundation::{
    MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSObject, NSObjectProtocol,
    NSString,
};
use once_cell::sync::OnceCell;

static URL_SENDER: OnceCell<Sender<String>> = OnceCell::new();

pub fn create_deep_link_receiver() -> Receiver<String> {
    let (tx, rx) = unbounded();
    URL_SENDER.set(tx).unwrap();
    rx
}

fn push_url(url: String) {
    if let Some(tx) = URL_SENDER.get() {
        let _ = tx.try_send(url);
    }
}

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
                // The direct object parameter has keyword '----' (keyDirectObject)
                // Use msg_send! because objc2-foundation doesn't expose this safely
                let param: Option<Retained<NSAppleEventDescriptor>> = unsafe {
                    msg_send![event, paramDescriptorForKeyword: 0x2d2d2d2d_u32]
                };

                if let Some(param) = param {
                    // Use msg_send! to get the string value
                    let url_str: Option<Retained<NSString>> = unsafe {
                        msg_send![&param, stringValue]
                    };

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

        // CRITICAL: NSAppleEventManager does NOT retain the event handler.
        // If `handler` goes out of scope and is deallocated, macOS will
        // attempt to message a dangling pointer when a URL event arrives,
        // causing a segmentation fault. We intentionally leak it here so
        // it lives for the entire duration of the application.
        std::mem::forget(handler);

        println!("Apple Event handler registered for deep links");
    }
}

/// Registers the Apple Event handler (macOS only).
#[cfg(target_os = "macos")]
pub use macos::register_delegate;

/// No-op on non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub fn register_delegate() {}

pub struct DeepLinkHandle {
    rx: Receiver<String>,
}

impl DeepLinkHandle {
    pub fn recv_blocking(&self) -> Result<String, async_channel::RecvError> {
        self.rx.recv_blocking()
    }
    pub fn try_recv(&self) -> Option<String> {
        self.rx.try_recv().ok()
    }
    pub async fn recv(&self) -> Option<String> {
        self.rx.recv().await.ok()
    }
}

pub fn create_deep_link_handle() -> DeepLinkHandle {
    let rx = create_deep_link_receiver();
    DeepLinkHandle { rx }
}
