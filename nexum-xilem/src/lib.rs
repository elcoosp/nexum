use async_channel::{unbounded, Receiver, Sender};
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::ProtocolObject;
#[cfg(target_os = "macos")]
use objc2::{define_class, msg_send, MainThreadOnly};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationDelegate};
#[cfg(target_os = "macos")]
use objc2_foundation::{MainThreadMarker, NSArray, NSObject, NSObjectProtocol, NSURL};
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
        #[name = "NexumAppDelegate"]
        struct AppDelegate;

        unsafe impl NSObjectProtocol for AppDelegate {}

        unsafe impl NSApplicationDelegate for AppDelegate {
            #[unsafe(method(application:openURLs:))]
            fn application_openURLs(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
                for url in urls.iter() {
                    if let Some(url_str) = url.absoluteString() {
                        let url_string = url_str.to_string();
                        push_url(url_string);
                    }
                }
            }
        }
    );

    impl AppDelegate {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            // Pass `mtm` to `alloc()` because the class is MainThreadOnly
            let this = Self::alloc(mtm).set_ivars(());
            unsafe { msg_send![super(this), init] }
        }
    }

    pub fn register_delegate() {
        let mtm = MainThreadMarker::new().expect("must be called on the main thread");
        let delegate = AppDelegate::new(mtm);
        let app = NSApplication::sharedApplication(mtm);
        app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    }
}

/// Registers the application delegate (macOS only).
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
