use async_channel::{Receiver, Sender, unbounded};

use crate::event::{DeepLinkEvent, Source};
use crate::handle::DeepLinkHandle;

/// The producer side of the deep-link transport.
///
/// Platform code (and application integrations) push URLs into the hub;
/// consumers receive them through a [`DeepLinkHandle`] obtained from
/// [`DeepLinkHub::handle`]. The transport is an unbounded multi-producer
/// multi-consumer channel: each event is delivered to exactly one consumer,
/// which makes cloning handles a convenient way to share the load between
/// workers.
#[derive(Debug)]
pub struct DeepLinkHub {
    tx: Sender<DeepLinkEvent>,
    rx: Receiver<DeepLinkEvent>,
}

impl DeepLinkHub {
    /// Creates a new hub with an unbounded event queue.
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
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
    pub fn push_url(&self, url: impl Into<String>) {
        self.push_event(DeepLinkEvent::new(url, Source::System));
    }

    /// Pushes a cold-start URL into the hub ([`Source::CommandLine`]).
    ///
    /// Use this for URLs discovered on the command line of the current
    /// process so applications can distinguish launch URLs from runtime
    /// hand-offs.
    pub fn push_launch_url(&self, url: impl Into<String>) {
        self.push_event(DeepLinkEvent::new(url, Source::CommandLine));
    }

    /// Pushes a pre-built event into the hub.
    pub fn push_event(&self, event: DeepLinkEvent) {
        // The hub itself owns a receiver, so the queue is connected for as
        // long as the hub lives; a failure can only come from a raw
        // `sender()` clone used after the hub was dropped, and dropping the
        // event then is the correct behavior.
        self.tx.try_send(event).ok();
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
    use crate::error::Result;
    use crate::event::Source;
    use crate::hub::DeepLinkHub;

    #[test]
    fn default_matches_new() {
        let _hub = DeepLinkHub::default();
    }

    #[test]
    fn push_url_delivers_system_sourced_event() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_url("myapp://runtime");
        let event = handle.recv_event_blocking().expect("event");
        assert_eq!(event.url, "myapp://runtime");
        assert_eq!(event.source, Source::System);
    }

    #[test]
    fn push_launch_url_delivers_command_line_sourced_event() {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.push_launch_url("myapp://cold-start");
        let event = handle.recv_event_blocking().expect("event");
        assert_eq!(event.source, Source::CommandLine);
    }

    #[test]
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
    fn sender_clone_pushes_events() -> Result<()> {
        let hub = DeepLinkHub::new();
        let handle = hub.handle();
        hub.sender()
            .try_send(crate::event::DeepLinkEvent::new(
                "myapp://via-sender",
                Source::System,
            ))
            .expect("channel is open");
        assert_eq!(handle.try_recv().as_deref(), Some("myapp://via-sender"));
        Ok(())
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
}
