use async_channel::{unbounded, Receiver, Sender};

pub struct Config {
    pub schemes: Vec<String>,
    pub app_links: Vec<AppLink>,
}

pub struct AppLink {
    pub host: String,
    pub path_prefixes: Vec<String>,
}

#[derive(Clone)]
pub struct DeepLinkHandle {
    rx: Receiver<String>,
}

impl DeepLinkHandle {
    /// Creates a new handle from a channel receiver.
    pub fn new(rx: Receiver<String>) -> Self {
        Self { rx }
    }

    pub fn recv_blocking(&self) -> Result<String, async_channel::RecvError> {
        self.rx.recv_blocking()
    }

    pub fn try_recv(&self) -> Option<String> {
        self.rx.try_recv().ok()
    }

    pub async fn recv(&self) -> Option<String> {
        self.rx.recv().await.ok()
    }

    pub fn drain_into(&self, out: &mut Vec<String>) {
        while let Some(url) = self.try_recv() {
            out.push(url);
        }
    }
}

pub struct DeepLinkHub {
    tx: Sender<String>,
    rx: Receiver<String>,
}

impl DeepLinkHub {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }

    pub fn push_url(&self, url: String) {
        let _ = self.tx.try_send(url);
    }

    pub fn handle(&self) -> DeepLinkHandle {
        DeepLinkHandle::new(self.rx.clone())
    }
}
