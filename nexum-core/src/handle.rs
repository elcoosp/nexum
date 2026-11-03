use async_channel::Receiver;

// `Error` is only referenced by the blocking receivers, which exist on
// native targets; the async transport itself never constructs it.
#[cfg(not(target_family = "wasm"))]
use crate::error::Error;
use crate::error::Result;
use crate::event::DeepLinkEvent;

/// The consumer side of the deep-link transport.
///
/// A handle wraps a clone of the hub's receiver. Cloning the handle creates
/// another consumer: events are distributed round-robin, exactly one
/// consumer receives each URL.
///
/// The handle works in blocking, async and `Stream` (behind the `stream`
/// feature) styles, so adapters can pick the model of their UI framework.
#[derive(Clone, Debug)]
pub struct DeepLinkHandle {
    rx: Receiver<DeepLinkEvent>,
}

impl DeepLinkHandle {
    /// Creates a handle from a raw event receiver.
    #[must_use]
    pub fn new(rx: Receiver<DeepLinkEvent>) -> Self {
        Self { rx }
    }

    /// Receives the next URL, waiting until one arrives.
    ///
    /// Returns `None` once the channel has been shut down (the hub and every
    /// producer were dropped).
    pub async fn recv(&self) -> Option<String> {
        self.recv_event().await.map(|event| event.url)
    }

    /// Receives the next full event, waiting until one arrives.
    ///
    /// Returns `None` once the channel has been shut down.
    pub async fn recv_event(&self) -> Option<DeepLinkEvent> {
        self.rx.recv().await.ok()
    }

    /// Receives the next URL and parses it into a [`url::Url`].
    ///
    /// Returns `None` once the channel has been shut down, and
    /// `Some(Err(_))` when the next URL is malformed (which must not block
    /// the consumer, so the error surfaces as an item).
    pub async fn recv_url(&self) -> Option<Result<url::Url>> {
        self.recv_event().await.map(|event| event.parse_url())
    }

    /// Blocking variant of [`DeepLinkHandle::recv`].
    ///
    /// Only available on native targets: the browser (`wasm`) has no
    /// blocking primitives, so consumption there goes through
    /// [`DeepLinkHandle::recv`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::HubClosed`] when the channel has been shut down.
    #[cfg(not(target_family = "wasm"))]
    pub fn recv_blocking(&self) -> Result<String> {
        self.recv_event_blocking().map(|event| event.url)
    }

    /// Blocking variant of [`DeepLinkHandle::recv_event`].
    ///
    /// Only available on native targets, like [`DeepLinkHandle::recv_blocking`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::HubClosed`] when the channel has been shut down.
    #[cfg(not(target_family = "wasm"))]
    pub fn recv_event_blocking(&self) -> Result<DeepLinkEvent> {
        self.rx.recv_blocking().map_err(|_| Error::HubClosed)
    }

    /// Receives the next URL if one is already queued, without waiting.
    ///
    /// Returns `None` when the queue is empty or the channel has been shut
    /// down; use [`DeepLinkHandle::recv_event_blocking`] to tell those cases
    /// apart.
    #[must_use]
    pub fn try_recv(&self) -> Option<String> {
        self.try_recv_event().map(|event| event.url)
    }

    /// Receives the next full event if one is already queued, without
    /// waiting.
    #[must_use]
    pub fn try_recv_event(&self) -> Option<DeepLinkEvent> {
        self.rx.try_recv().ok()
    }

    /// Drains every currently queued URL into a vector.
    #[must_use]
    pub fn drain(&self) -> Vec<String> {
        self.drain_events()
            .into_iter()
            .map(|event| event.url)
            .collect()
    }

    /// Drains every currently queued event into a vector.
    #[must_use]
    pub fn drain_events(&self) -> Vec<DeepLinkEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.try_recv_event() {
            events.push(event);
        }
        events
    }

    /// Converts the handle into a [`futures_core::Stream`] of URLs.
    ///
    /// Only available with the `stream` feature.
    #[cfg(feature = "stream")]
    #[must_use]
    pub fn into_stream(self) -> DeepLinkStream {
        DeepLinkStream {
            rx: Box::pin(self.rx),
        }
    }
}

/// A [`futures_core::Stream`] adapter over [`DeepLinkHandle`], yielding the
/// raw URL of every event. The stream terminates when the channel shuts
/// down.
///
/// Only available with the `stream` feature.
#[cfg(feature = "stream")]
#[derive(Debug)]
pub struct DeepLinkStream {
    rx: std::pin::Pin<Box<Receiver<DeepLinkEvent>>>,
}

#[cfg(feature = "stream")]
impl futures_core::Stream for DeepLinkStream {
    type Item = String;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.get_mut()
            .rx
            .as_mut()
            .poll_next(cx)
            .map(|item| item.map(|event| event.url))
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    // Only the native-gated tests touch the error types directly.
    #[cfg(not(target_family = "wasm"))]
    use crate::error::{Error, Result};
    use crate::event::{DeepLinkEvent, Source};
    use crate::hub::DeepLinkHub;

    #[test]
    fn new_adopts_foreign_receiver() {
        let (tx, rx) = async_channel::unbounded();
        tx.try_send(DeepLinkEvent::new("myapp://x", Source::System))
            .expect("send");
        let handle = crate::handle::DeepLinkHandle::new(rx);
        assert_eq!(handle.try_recv().as_deref(), Some("myapp://x"));
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn recv_event_blocking_delivers_events() -> Result<()> {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("myapp://1");
        hub.push_launch_url("myapp://2");
        let first = handle.recv_event_blocking()?;
        let second = handle.recv_event_blocking()?;
        assert_eq!(first.url, "myapp://1");
        assert_eq!(second.url, "myapp://2");
        assert_eq!(second.source, Source::CommandLine);
        Ok(())
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn recv_blocking_reports_closed_channel() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        drop(hub);
        let error = handle.recv_blocking().expect_err("closed");
        assert!(matches!(error, Error::HubClosed));
    }

    #[test]
    fn async_recv_returns_none_after_shutdown() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("myapp://last");
        drop(hub);
        assert_eq!(block_on(handle.recv()).as_deref(), Some("myapp://last"));
        assert_eq!(block_on(handle.recv()), None);
    }

    #[test]
    fn recv_url_parses_and_reports_malformed_payloads() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("not a url");
        let error = block_on(handle.recv_url())
            .expect("item")
            .expect_err("malformed");
        assert!(matches!(error, crate::error::Error::MalformedUrl(raw, _) if raw == "not a url"));
        hub.push_url("myapp://ok");
        let url = block_on(handle.recv_url()).expect("item").expect("parsed");
        assert_eq!(url.scheme(), "myapp");
    }

    #[test]
    fn try_recv_is_none_when_empty() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        assert_eq!(handle.try_recv(), None);
        assert_eq!(handle.try_recv_event(), None);
        hub.push_url("myapp://1");
        assert_eq!(handle.try_recv().as_deref(), Some("myapp://1"));
    }

    #[test]
    fn drain_empties_the_queue() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("myapp://1");
        hub.push_url("myapp://2");
        assert_eq!(handle.drain(), vec!["myapp://1", "myapp://2"]);
        assert_eq!(handle.drain().len(), 0);
    }

    #[test]
    fn drain_events_keeps_metadata() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_launch_url("myapp://cold");
        let events = handle.drain_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, Source::CommandLine);
    }

    #[cfg(feature = "stream")]
    #[test]
    fn stream_yields_urls_and_terminates() {
        use futures::StreamExt;

        let hub = DeepLinkHub::new();
        let handle = hub.handle().into_stream();
        hub.push_url("myapp://1");
        hub.push_url("myapp://2");
        drop(hub);
        let urls: Vec<String> = block_on(handle.collect());
        assert_eq!(urls, vec!["myapp://1", "myapp://2"]);
    }
}
