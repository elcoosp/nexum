use async_channel::{Receiver, Sender, TrySendError, unbounded};

use crate::error::Error;
use crate::event::{DeepLinkEvent, Source};
use crate::handle::DeepLinkHandle;

/// The producer side of the deep-link transport.
///
/// Platform code (and application integrations) push URLs into the hub;
/// consumers receive them through a [`DeepLinkHandle`] obtained from
/// [`DeepLinkHub::handle`]. The transport is a multi-producer
/// multi-consumer channel: each event is delivered to exactly one consumer,
/// which makes cloning handles a convenient way to share the load between
/// workers.
///
/// The queue is unbounded by default — deep links are rare, human-driven
/// events. Applications that must never buffer more than a fixed number of
/// URLs can use [`DeepLinkHub::bounded`]; pushes then block (or fail with
/// [`Error::QueueFull`] for the non-blocking variants) while the queue is
/// full.
#[derive(Debug)]
pub struct DeepLinkHub {
    tx: Sender<DeepLinkEvent>,
    rx: Receiver<DeepLinkEvent>,
    capacity: Option<usize>,
}

impl DeepLinkHub {
    /// Creates a new hub with an unbounded event queue.
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            tx,
            rx,
            capacity: None,
        }
    }

    /// Creates a new hub whose event queue holds at most `capacity` events.
    ///
    /// On a bounded hub the push methods block the calling thread while the
    /// queue is full — pushers are expected to run on dedicated threads
    /// (platform callbacks, relay bridges), never on the UI thread.
    /// Use [`DeepLinkHub::try_push_url`] when blocking is unacceptable.
    #[must_use]
    pub fn bounded(capacity: usize) -> Self {
        let (tx, rx) = async_channel::bounded(capacity.max(1));
        Self {
            tx,
            rx,
            capacity: Some(capacity.max(1)),
        }
    }

    /// Returns the queue capacity, or `None` when the queue is unbounded.
    #[must_use]
    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }

    /// Returns a clone of the raw producer channel.
    ///
    /// This is the escape hatch for integrations that want to talk to the
    /// queue directly (e.g. framework bridges). Events pushed through it use
    /// [`Source::System`] by default when constructed with
    /// [`DeepLinkEvent::new`].
    #[must_use]
    pub fn sender(&self) -> Sender<DeepLinkEvent> {
        self.tx.clone()
    }

    /// Pushes a runtime URL into the hub ([`Source::System`]).
    ///
    /// On a [bounded](DeepLinkHub::bounded) hub this blocks the calling
    /// thread while the queue is full.
    pub fn push_url(&self, url: impl Into<String>) {
        self.push_event(DeepLinkEvent::new(url, Source::System));
    }

    /// Pushes a cold-start URL into the hub ([`Source::CommandLine`]).
    ///
    /// Use this for URLs discovered on the command line of the current
    /// process so applications can distinguish launch URLs from runtime
    /// hand-offs.
    ///
    /// On a [bounded](DeepLinkHub::bounded) hub this blocks the calling
    /// thread while the queue is full.
    pub fn push_launch_url(&self, url: impl Into<String>) {
        self.push_event(DeepLinkEvent::new(url, Source::CommandLine));
    }

    /// Non-blocking variant of [`DeepLinkHub::push_url`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::QueueFull`] when the bounded queue is full, and
    /// [`Error::HubClosed`] when the hub (and every producer) has been
    /// dropped. On an unbounded hub only [`Error::HubClosed`] is possible.
    pub fn try_push_url(&self, url: impl Into<String>) -> Result<(), Error> {
        self.try_push_event(DeepLinkEvent::new(url, Source::System))
    }

    /// Non-blocking variant of [`DeepLinkHub::push_launch_url`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::QueueFull`] when the bounded queue is full, and
    /// [`Error::HubClosed`] when the hub (and every producer) has been
    /// dropped. On an unbounded hub only [`Error::HubClosed`] is possible.
    pub fn try_push_launch_url(&self, url: impl Into<String>) -> Result<(), Error> {
        self.try_push_event(DeepLinkEvent::new(url, Source::CommandLine))
    }

    /// Pushes a pre-built event into the hub.
    ///
    /// On a [bounded](DeepLinkHub::bounded) hub this blocks the calling
    /// thread while the queue is full; on an unbounded hub it never blocks.
    /// On `wasm` targets, where threads (and therefore blocking) do not
    /// exist, the bounded push degrades to a non-blocking attempt that drops
    /// the event while the queue is full.
    pub fn push_event(&self, event: DeepLinkEvent) {
        match self.capacity {
            // Bounded: wait for room instead of silently dropping the URL.
            Some(_) => self.push_bounded(event),
            // Unbounded: the hub itself owns a receiver, so the queue is
            // connected for as long as the hub lives; a failure can only
            // come from a raw `sender()` clone used after the hub was
            // dropped, and dropping the event then is the correct behavior.
            None => {
                self.tx.try_send(event).ok();
            }
        }
    }

    /// Bounded push: waits for room where blocking is possible.
    #[cfg(not(target_family = "wasm"))]
    fn push_bounded(&self, event: DeepLinkEvent) {
        self.tx.send_blocking(event).ok();
    }

    /// Bounded push on the web: no threads to park, so the event is dropped
    /// while the queue is full.
    #[cfg(target_family = "wasm")]
    fn push_bounded(&self, event: DeepLinkEvent) {
        self.tx.try_send(event).ok();
    }

    /// Non-blocking variant of [`DeepLinkHub::push_event`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::QueueFull`] when the bounded queue is full, and
    /// [`Error::HubClosed`] when the hub (and every producer) has been
    /// dropped. On an unbounded hub only [`Error::HubClosed`] is possible.
    pub fn try_push_event(&self, event: DeepLinkEvent) -> Result<(), Error> {
        match self.tx.try_send(event) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(Error::QueueFull),
            Err(TrySendError::Closed(_)) => Err(Error::HubClosed),
        }
    }

    /// Creates a new consumer handle for this hub.
    #[must_use]
    pub fn handle(&self) -> DeepLinkHandle {
        DeepLinkHandle::new(self.rx.clone())
    }
}

impl Default for DeepLinkHub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::event::Source;
    use crate::hub::DeepLinkHub;

    #[test]
    fn default_matches_new() {
        let _hub = DeepLinkHub::default();
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn push_url_delivers_system_sourced_event() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("myapp://runtime");
        let event = handle.recv_event_blocking().expect("event");
        assert_eq!(event.url, "myapp://runtime");
        assert_eq!(event.source, Source::System);
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn push_launch_url_delivers_command_line_sourced_event() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_launch_url("myapp://cold-start");
        let event = handle.recv_event_blocking().expect("event");
        assert_eq!(event.source, Source::CommandLine);
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn push_event_keeps_original_source() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_event(crate::event::DeepLinkEvent::new(
            "myapp://custom",
            Source::CommandLine,
        ));
        let event = handle.recv_event_blocking().expect("event");
        assert_eq!(event.url, "myapp://custom");
        assert_eq!(event.source, Source::CommandLine);
    }

    #[test]
    fn handles_share_the_queue_round_robin() {
        let hub = DeepLinkHub::new();
        let first = hub.handle();
        let second = hub.handle();
        hub.push_url("myapp://1");
        hub.push_url("myapp://2");
        // Exactly one of the two handles receives each event.
        let seen_first = first.drain();
        let seen_second = second.drain();
        let mut all = seen_first;
        all.extend(seen_second);
        all.sort();
        assert_eq!(all, vec!["myapp://1", "myapp://2"]);
    }

    #[test]
    fn sender_clone_pushes_events() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.sender()
            .try_send(crate::event::DeepLinkEvent::new(
                "myapp://via-sender",
                Source::System,
            ))
            .expect("channel is open");
        assert_eq!(handle.try_recv().as_deref(), Some("myapp://via-sender"));
    }

    #[test]
    fn push_after_queue_shutdown_is_dropped_silently() {
        let hub = DeepLinkHub::new();
        let sender = hub.sender();
        drop(hub);
        // No receivers remain: the event is dropped instead of panicking.
        sender
            .try_send(crate::event::DeepLinkEvent::new(
                "myapp://lost",
                Source::System,
            ))
            .expect_err("channel disconnected");
    }

    #[test]
    fn bounded_hub_reports_its_capacity() {
        assert_eq!(DeepLinkHub::new().capacity(), None);
        assert_eq!(DeepLinkHub::bounded(4).capacity(), Some(4));
        // A capacity of zero is meaningless; it is clamped to one slot.
        assert_eq!(DeepLinkHub::bounded(0).capacity(), Some(1));
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn try_push_reports_queue_full_on_a_bounded_hub() {
        let hub = DeepLinkHub::bounded(2);
        let handle = hub.handle();

        hub.try_push_url("myapp://1").expect("room for 1");
        hub.try_push_launch_url("myapp://2").expect("room for 2");
        assert!(matches!(
            hub.try_push_url("myapp://3"),
            Err(crate::error::Error::QueueFull)
        ));

        // Draining one event makes room for exactly one more.
        handle.recv_event_blocking().expect("event");
        hub.try_push_url("myapp://3").expect("room again");
    }

    #[test]
    fn try_push_reports_closed_hub() {
        let hub = DeepLinkHub::bounded(2);
        // Closing the receiver side (private field, test-only access)
        // disconnects the channel without dropping the hub.
        hub.rx.close();
        assert!(matches!(
            hub.try_push_url("myapp://late"),
            Err(crate::error::Error::HubClosed)
        ));
        assert!(matches!(
            hub.try_push_launch_url("myapp://late"),
            Err(crate::error::Error::HubClosed)
        ));
        // The blocking push degrades to a silent no-op on a closed channel.
        hub.push_url("myapp://dropped");
    }

    #[test]
    fn unbounded_try_push_never_reports_queue_full() {
        let hub = DeepLinkHub::new();
        for i in 0..10 {
            hub.try_push_url(format!("myapp://{i}")).expect("unbounded");
        }
        assert_eq!(hub.handle().drain().len(), 10);
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn blocking_push_waits_for_room_on_a_bounded_hub() {
        use std::sync::Arc;
        use std::time::Duration;

        let hub = Arc::new(DeepLinkHub::bounded(1));
        let handle = hub.handle();
        hub.push_url("myapp://fills-queue");

        // The queue is full: this push must block until the consumer drains
        // the first event, then succeed.
        let producer = {
            let hub = Arc::clone(&hub);
            std::thread::spawn(move || hub.push_url("myapp://waits"))
        };
        std::thread::sleep(Duration::from_millis(50));
        // The producer is still parked on the full queue...
        assert!(hub.try_push_url("myapp://still-full").is_err());

        let first = handle.recv_event_blocking().expect("first event");
        assert_eq!(first.url, "myapp://fills-queue");

        producer.join().expect("producer finished");
        let second = handle.recv_event_blocking().expect("second event");
        assert_eq!(second.url, "myapp://waits");
    }
}
