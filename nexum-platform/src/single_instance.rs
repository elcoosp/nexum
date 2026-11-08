//! Single-instance hand-off over a local socket.
//!
//! When an application is launched a second time (typically because the
//! operating system opened a deep link in a new process), the new process
//! can forward its URLs to the already-running instance through a local
//! socket and exit instead of starting a second GUI:
//!
//! ```no_run
//! use nexum_core::Config;
//! use nexum_platform::single_instance::{self, Acquisition};
//!
//! # fn run_app() {}
//! # fn main() -> nexum_core::Result<()> {
//! let config = Config::builder().scheme("myapp").build().expect("valid config");
//!
//! match single_instance::acquire(&config, &[])? {
//!     Acquisition::Primary(_guard) => {
//!         // This process owns the socket: every URL a second instance
//!         // forwards arrives in the hub as a normal deep-link event.
//!         // Keep `_guard` alive and run the application.
//!         run_app();
//!     }
//!     Acquisition::HandedOff { forwarded } => {
//!         // Another instance is running; our URLs were delivered to it.
//!         println!("forwarded {forwarded} URL(s); exiting");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The transport is a Unix domain socket on Linux/macOS (and the other unix
//! targets) and a loopback TCP socket on Windows; both speak the same tiny
//! line protocol (`NEXUM/1`), so no third-party IPC crate is involved.
//! Peers that do not answer with the nexum acknowledgement make the
//! hand-off fail with an I/O error, so an unrelated service squatting on
//! the Windows port range is reported instead of silently swallowed.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
use std::time::Duration;

use nexum_core::{Config, Error, Result};

use crate::push_url;

/// Magic protocol header: `NEXUM/1 <url-count>\n` followed by one URL per
/// line, terminated by an `OK\n` acknowledgement from the primary.
const MAGIC: &str = "NEXUM/1";
const ACK: &[u8] = b"OK\n";

/// How long a connection may take to complete its handshake.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);

/// How often the accept loop re-checks the shutdown signal.
const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[cfg(unix)]
type Listener = std::os::unix::net::UnixListener;
#[cfg(windows)]
type Listener = std::net::TcpListener;

/// The local socket identity derived from the configuration.
///
/// Two processes that register the same schemes compute the same name,
/// which is what makes the hand-off find the running instance.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint {
    kind: EndpointKind,
}

#[cfg(unix)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum EndpointKind {
    /// Unix domain socket file name, resolved inside a runtime directory.
    UnixSocket { file: String },
}

#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum EndpointKind {
    /// Windows loopback port.
    TcpPort { port: u16 },
}

impl Endpoint {
    /// Derives the endpoint from the configured schemes.
    ///
    /// FNV-1a is used because it is deterministic across processes and
    /// platforms and needs no dependency. The schemes are the only input, so
    /// two applications collide only when they register the exact same
    /// schemes — in which case they *are* the same app for deep-link
    /// purposes.
    fn from_config(config: &Config) -> Self {
        let mut hash: u32 = 0x811c_9dc5;
        for scheme in &config.schemes {
            for byte in scheme.to_ascii_lowercase().as_bytes() {
                hash ^= u32::from(*byte);
                hash = hash.wrapping_mul(0x0100_0193);
            }
        }
        #[cfg(unix)]
        {
            Self {
                kind: EndpointKind::UnixSocket {
                    file: format!("nexum-{hash:08x}.sock"),
                },
            }
        }
        #[cfg(windows)]
        {
            Self {
                kind: EndpointKind::TcpPort {
                    // 47xxx is an unregistered port range; the protocol
                    // handshake below rejects peers that are not nexum
                    // endpoints.
                    port: 47_000 + (hash % 1_000) as u16,
                },
            }
        }
    }
}

/// Encodes a hand-off request: the magic header, the URL count and one URL
/// per line.
fn encode_request(urls: &[String]) -> Vec<u8> {
    let mut request = format!("{MAGIC} {}\n", urls.len()).into_bytes();
    for url in urls {
        // URLs never contain raw newlines (RFC 3986), but stay defensive so
        // the framing cannot be corrupted by a hand-crafted argument.
        request.extend_from_slice(url.replace(['\n', '\r'], " ").as_bytes());
        request.push(b'\n');
    }
    request
}

/// Parses the request header. Returns the announced URL count, or `None`
/// when the magic does not match — foreign peers on the same port are
/// rejected this way.
fn parse_header(header: &str) -> Option<usize> {
    let (magic, count) = header.trim_end().split_once(' ')?;
    (magic == MAGIC).then(|| count.parse().ok())?
}

/// Resolves the directory the Unix socket file is created in.
#[cfg(unix)]
fn runtime_directory() -> PathBuf {
    runtime_directory_from(std::env::var("XDG_RUNTIME_DIR").ok(), &std::env::temp_dir())
}

/// Pure core of [`runtime_directory`] so both branches stay testable
/// without mutating the process environment.
#[cfg(unix)]
fn runtime_directory_from(xdg: Option<String>, fallback: &Path) -> PathBuf {
    match xdg {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => fallback.to_path_buf(),
    }
}

/// Outcome of trying to acquire the single-instance endpoint.
#[derive(Debug)]
#[must_use = "dropping the primary guard immediately closes the socket again"]
pub enum Acquisition {
    /// This process is now the primary: it owns the local socket and
    /// forwards every URL it receives into the global hub. The
    /// [`PrimaryGuard`] must be kept alive for the process' lifetime.
    Primary(PrimaryGuard),
    /// Another instance answered; `forwarded` URLs were delivered to it and
    /// this process should exit.
    HandedOff {
        /// Number of URLs the running instance accepted.
        forwarded: usize,
    },
}

/// Keeps the single-instance socket alive. Dropping it signals the accept
/// thread to end (within one poll interval) and removes the Unix socket
/// file, so the next launch can become the primary again.
#[derive(Debug)]
pub struct PrimaryGuard {
    #[cfg(unix)]
    path: PathBuf,
    shutdown: std::sync::mpsc::Sender<()>,
}

impl Drop for PrimaryGuard {
    fn drop(&mut self) {
        // Signalling shutdown ends the accept loop, which then drops the
        // listener and closes the socket.
        let _ = self.shutdown.send(());
        #[cfg(unix)]
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Becomes the single-instance primary or hands the given URLs off to the
/// already-running instance.
///
/// Must be called after [`crate::init`]/[`crate::set_hub`]: the primary
/// forwards every URL it receives on the socket into the global hub.
///
/// # Errors
///
/// Returns [`Error::HubNotSet`] when no hub is initialized, and
/// [`Error::Platform`] / [`Error::Io`] when the local socket cannot be
/// created or the running instance does not speak the nexum protocol.
pub fn acquire(config: &Config, urls: &[String]) -> Result<Acquisition> {
    #[cfg(unix)]
    let dir = runtime_directory();
    #[cfg(windows)]
    let dir = std::env::temp_dir();
    acquire_in(&dir, config, urls)
}

/// [`acquire`] with an explicit runtime directory (used by tests).
pub(crate) fn acquire_in(dir: &Path, config: &Config, urls: &[String]) -> Result<Acquisition> {
    if crate::hub().is_none() {
        return Err(Error::HubNotSet);
    }
    let endpoint = Endpoint::from_config(config);

    // Bind first: this fails with an address-in-use error when another
    // instance of the same app already owns the endpoint.
    match bind(&endpoint, dir) {
        Ok(listener) => Ok(Acquisition::Primary(PrimaryGuard::spawn(
            listener, &endpoint, dir,
        ))),
        Err(BindError::InUse) => {
            // The endpoint is owned by somebody: hand the URLs over.
            hand_off(&endpoint, dir, urls).map(|forwarded| Acquisition::HandedOff { forwarded })
        }
        Err(BindError::Io(error)) => Err(Error::Io(error)),
    }
}

#[derive(Debug)]
enum BindError {
    InUse,
    Io(std::io::Error),
}

#[cfg(unix)]
fn bind(endpoint: &Endpoint, dir: &Path) -> std::result::Result<Listener, BindError> {
    // `EndpointKind` has exactly one variant per platform, so this pattern
    // is irrefutable there.
    let EndpointKind::UnixSocket { file } = &endpoint.kind;
    match Listener::bind(dir.join(file)) {
        Ok(listener) => Ok(listener),
        // On Unix a *stale* socket file from a crashed instance also
        // surfaces as `AddrInUse`; the caller falls back to the hand-off,
        // whose connect() fails, and the stale file is then reported.
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => Err(BindError::InUse),
        Err(error) => Err(BindError::Io(error)),
    }
}

#[cfg(windows)]
fn bind(endpoint: &Endpoint, _dir: &Path) -> std::result::Result<Listener, BindError> {
    let EndpointKind::TcpPort { port } = &endpoint.kind;
    Listener::bind(("127.0.0.1", *port)).map_err(|error| match error.kind() {
        std::io::ErrorKind::AddrInUse => BindError::InUse,
        _ => BindError::Io(error),
    })
}

/// Sends the URLs to the running instance and waits for its acknowledgement.
#[cfg(unix)]
fn hand_off(endpoint: &Endpoint, dir: &Path, urls: &[String]) -> Result<usize> {
    let EndpointKind::UnixSocket { file } = &endpoint.kind;
    let stream = std::os::unix::net::UnixStream::connect(dir.join(file)).map_err(|error| {
        // The endpoint exists (bind reported it in use) but nobody answers:
        // a stale socket file from a crashed instance.
        if error.kind() == std::io::ErrorKind::ConnectionRefused {
            Error::Platform("stale single-instance socket: the previous instance crashed".into())
        } else {
            Error::Io(error)
        }
    })?;
    // A missing acknowledgement must surface as an error instead of
    // blocking the second instance forever.
    let _ = stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT));
    let _ = stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT));
    exchange(stream, urls)
}

#[cfg(windows)]
fn hand_off(endpoint: &Endpoint, _dir: &Path, urls: &[String]) -> Result<usize> {
    let EndpointKind::TcpPort { port } = &endpoint.kind;
    let stream = std::net::TcpStream::connect(("127.0.0.1", *port))?;
    let _ = stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT));
    let _ = stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT));
    exchange(stream, urls)
}

/// Writes the request, reads the acknowledgement and returns the number of
/// URLs the primary accepted.
fn exchange(stream: impl Read + Write, urls: &[String]) -> Result<usize> {
    let mut stream = stream;
    stream.write_all(&encode_request(urls))?;
    stream.flush()?;
    let mut ack = [0_u8; ACK.len()];
    let read = stream.read_exact(&mut ack).map_err(|_| {
        Error::Platform("the running instance did not acknowledge the hand-off".into())
    });
    read?;
    if ack != ACK {
        return Err(Error::Platform(
            "the running instance did not acknowledge the hand-off".into(),
        ));
    }
    Ok(urls.len())
}

/// The connection handler of the primary: read one request, deliver every
/// URL through `deliver` and acknowledge.
fn serve(stream: impl Read + Write, deliver: &mut dyn FnMut(String)) {
    let mut reader = BufReader::new(stream);
    let mut header = String::new();
    if reader.read_line(&mut header).is_err() || header.is_empty() {
        return;
    }
    let Some(count) = parse_header(&header) else {
        return;
    };
    let mut urls = Vec::new();
    for _ in 0..count {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => return, // client vanished mid-request
            // Only newline-terminated lines count: a final line without
            // `\n` means the request was truncated and must not be
            // delivered (nor acknowledged).
            Ok(_) if line.ends_with('\n') => urls.push(line.trim_end().to_owned()),
            Ok(_) => return,
        }
    }
    for url in &urls {
        deliver(url.clone());
    }
    let _ = reader.get_mut().write_all(ACK);
}

impl PrimaryGuard {
    /// Spawns the accept thread for a freshly bound listener.
    fn spawn(mut listener: Listener, endpoint: &Endpoint, dir: &Path) -> Self {
        #[cfg(unix)]
        let EndpointKind::UnixSocket { file } = &endpoint.kind;
        #[cfg(windows)]
        let _ = endpoint;
        let _ = dir;
        let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();
        std::thread::Builder::new()
            .name("nexum-single-instance".to_owned())
            .spawn(move || {
                let _ = listener.set_nonblocking(true);
                let mut deliver = |url| {
                    // A missing hub (the application is shutting down)
                    // drops the URL: there is nobody left to deliver it to.
                    let _ = push_url(url);
                };
                // Stop when the guard signals shutdown or the listener
                // dies; both are folded into the loop condition so there is
                // no unreachable branch left for the coverage gate.
                while accept_once(&mut listener, &mut deliver) && shutdown_rx.try_recv().is_err() {}
            })
            .expect("nexum-single-instance: failed to spawn the accept thread");
        Self {
            #[cfg(unix)]
            path: dir.join(file),
            shutdown: shutdown_tx,
        }
    }
}

/// Handles at most one connection on the listener. Returns `false` when the
/// listener is gone (any accept error other than "no pending connection")
/// and the accept thread should end.
fn accept_once(listener: &mut Listener, deliver: &mut dyn FnMut(String)) -> bool {
    match listener.accept() {
        Ok((stream, _)) => {
            let _ = stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT));
            let _ = stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT));
            serve(stream, deliver);
            true
        }
        // No pending connection: idle briefly, then re-check the shutdown
        // signal in the caller's loop.
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
            std::thread::sleep(ACCEPT_POLL_INTERVAL);
            true
        }
        Err(_) => false,
    }
}

/// Test helper: unwraps a [`Acquisition::Primary`], panicking otherwise.
#[cfg(test)]
pub(crate) fn expect_primary(acquisition: Acquisition) -> PrimaryGuard {
    match acquisition {
        Acquisition::Primary(guard) => guard,
        Acquisition::HandedOff { forwarded } => {
            panic!("expected the primary, got a hand-off of {forwarded} URL(s)")
        }
    }
}

/// Test helper: unwraps a [`Acquisition::HandedOff`], returning the count of
/// forwarded URLs.
#[cfg(all(test, unix))]
pub(crate) fn expect_handoff(acquisition: &Acquisition) -> usize {
    match acquisition {
        Acquisition::HandedOff { forwarded } => *forwarded,
        Acquisition::Primary(_) => panic!("expected a hand-off, got the primary"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_is_deterministic_and_scheme_insensitive() {
        let lower = Config::new(["myapp"]);
        let upper = Config::new(["MYAPP"]);
        let other = Config::new(["otherapp"]);
        assert_eq!(Endpoint::from_config(&lower), Endpoint::from_config(&upper));
        assert_ne!(Endpoint::from_config(&lower), Endpoint::from_config(&other));
    }

    #[test]
    fn serve_delivers_urls_and_acknowledges() {
        let urls = vec!["myapp://a".to_owned(), "myapp://b".to_owned()];
        let mut delivered = Vec::new();
        let mut request = encode_request(&urls);
        request.extend_from_slice(b"trailing bytes are ignored");

        let mut stream = std::io::Cursor::new(request.clone());
        serve(&mut stream, &mut |url| delivered.push(url));
        let response = stream.into_inner();

        // The acknowledgement directly follows the received bytes.
        assert_eq!(delivered, urls);
        assert_eq!(&response[request.len()..], ACK);
    }

    #[test]
    fn serve_ignores_foreign_and_truncated_requests() {
        let delivered = std::cell::RefCell::new(Vec::new());
        let mut deliver = |url: String| delivered.borrow_mut().push(url);

        // Sanity anchor: a valid request delivers through the same closure.
        let mut stream = std::io::Cursor::new(encode_request(&["myapp://ok".to_owned()]));
        serve(&mut stream, &mut deliver);
        assert_eq!(*delivered.borrow(), ["myapp://ok".to_owned()]);

        // A foreign protocol (no magic) gets no acknowledgement.
        let request = b"GET / HTTP/1.1\n".to_vec();
        let mut stream = std::io::Cursor::new(request.clone());
        serve(&mut stream, &mut deliver);
        assert_eq!(stream.into_inner(), request, "reply untouched");

        // A client that disappears mid-request also gets nothing.
        let mut truncated = encode_request(&["myapp://a".to_owned(), "myapp://b".to_owned()]);
        truncated.truncate(truncated.len() - 2);
        let mut stream = std::io::Cursor::new(truncated.clone());
        serve(&mut stream, &mut deliver);
        assert_eq!(stream.into_inner(), truncated, "reply untouched");

        // Neither malformed request delivered anything extra.
        assert_eq!(*delivered.borrow(), ["myapp://ok".to_owned()]);
    }

    #[test]
    fn serve_rejects_an_empty_connection_and_a_vanished_client() {
        let delivered = std::cell::RefCell::new(Vec::new());
        let mut deliver = |url: String| delivered.borrow_mut().push(url);

        // Sanity anchor: a valid request delivers through the same closure.
        let mut stream = std::io::Cursor::new(encode_request(&["myapp://ok".to_owned()]));
        serve(&mut stream, &mut deliver);
        assert_eq!(*delivered.borrow(), ["myapp://ok".to_owned()]);

        // An empty stream (connection closed before any byte) is ignored.
        let mut stream = std::io::Cursor::new(Vec::new());
        serve(&mut stream, &mut deliver);
        assert_eq!(stream.into_inner(), Vec::<u8>::new());

        // The client announced two URLs but vanished before sending the
        // second line: nothing is delivered, nothing acknowledged.
        let mut request = encode_request(&["myapp://a".to_owned(), "myapp://b".to_owned()]);
        let cut = request.len() - "myapp://b\n".len();
        request.truncate(cut);
        let mut stream = std::io::Cursor::new(request.clone());
        serve(&mut stream, &mut deliver);
        assert_eq!(stream.into_inner(), request, "reply untouched");
        assert_eq!(*delivered.borrow(), ["myapp://ok".to_owned()]);
    }

    #[cfg(unix)]
    #[test]
    fn accept_once_reports_a_dead_listener() {
        use std::io::Write as _;
        use std::os::unix::io::FromRawFd;

        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("live.sock");
        let mut live = Listener::bind(&path).expect("bind");
        // A client queues a request so the live accept delivers one URL.
        let mut client = std::os::unix::net::UnixStream::connect(&path).expect("connect");
        client
            .write_all(&encode_request(&["myapp://live".to_owned()]))
            .expect("write request");

        let delivered = std::cell::RefCell::new(Vec::new());
        let mut deliver = |url: String| delivered.borrow_mut().push(url);
        assert!(
            accept_once(&mut live, &mut deliver),
            "live listener continues"
        );
        assert_eq!(*delivered.borrow(), ["myapp://live".to_owned()]);

        // SAFETY: fd -2 is never a valid descriptor, so `accept` fails with
        // EBADF deterministically and the placeholder's `close(-2)` is a
        // harmless no-op (std only reserves -1).
        let mut dead = unsafe { std::os::unix::net::UnixListener::from_raw_fd(-2) };
        let keep_going = accept_once(&mut dead, &mut deliver);
        // Leak the placeholder: dropping it would close the fake descriptor,
        // which trips std's IO-safety abort.
        std::mem::forget(dead);
        assert!(!keep_going, "a dead listener ends the accept loop");
        assert_eq!(*delivered.borrow(), ["myapp://live".to_owned()]);
    }

    #[cfg(unix)]
    #[test]
    fn exchange_over_a_real_socket_fails_without_an_ack() {
        // Covers the `UnixStream` monomorphization of `exchange`, including
        // the error path: the server accepts but closes without ever
        // acknowledging, so the client's read fails.
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("unacked.sock");
        let listener = std::os::unix::net::UnixListener::bind(&path).expect("bind");
        let client = std::os::unix::net::UnixStream::connect(&path).expect("connect");
        let (server, _) = listener.accept().expect("accept");

        let exchange_thread = std::thread::spawn(move || {
            exchange(client, &["myapp://x".to_owned()]).expect_err("the peer never acknowledged")
        });
        // Give the client time to write its request and park on the read,
        // then close the server without acknowledging.
        std::thread::sleep(std::time::Duration::from_millis(50));
        drop(server);
        let error = exchange_thread.join().expect("exchange thread");
        assert!(matches!(error, Error::Platform(_)));
    }

    #[cfg(unix)]
    #[test]
    fn hand_off_maps_unusual_connect_errors_to_io() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // A socket path beyond the 108-byte sun_path limit makes
        // `UnixStream::connect` fail with an `InvalidInput` error before any
        // syscall — reported as `Error::Io`, not as a stale socket.
        let endpoint = Endpoint {
            kind: EndpointKind::UnixSocket {
                file: "n".repeat(200),
            },
        };
        let error = hand_off(&endpoint, tmp.path(), &[]).expect_err("path too long");
        assert!(matches!(error, Error::Io(_)));
    }

    #[cfg(unix)]
    #[test]
    fn runtime_directory_reads_the_environment() {
        // Whatever the environment holds, the result is a usable path.
        assert!(!runtime_directory().as_os_str().is_empty());
    }

    #[test]
    #[should_panic(expected = "expected the primary, got a hand-off")]
    fn expect_primary_rejects_a_handoff() {
        drop(expect_primary(Acquisition::HandedOff { forwarded: 0 }));
    }

    #[cfg(unix)]
    #[test]
    #[should_panic(expected = "expected a hand-off, got the primary")]
    fn expect_handoff_rejects_a_primary() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = Config::new(["nexum-panic-helper"]);
        let endpoint = Endpoint::from_config(&config);
        let listener = Listener::bind(tmp.path().join("helper.sock")).expect("bind");
        let guard = PrimaryGuard::spawn(listener, &endpoint, tmp.path());
        expect_handoff(&Acquisition::Primary(guard));
    }

    #[test]
    fn exchange_accepts_an_ack_and_rejects_other_replies() {
        /// In-memory duplex stream for protocol tests.
        struct Channel {
            incoming: std::io::Cursor<Vec<u8>>,
            outgoing: Vec<u8>,
        }
        impl Read for Channel {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.incoming.read(buf)
            }
        }
        impl Write for Channel {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.outgoing.extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        // The happy path encodes the request into the outgoing buffer and
        // reports the forwarded count.
        let mut channel = Channel {
            incoming: std::io::Cursor::new(ACK.to_vec()),
            outgoing: Vec::new(),
        };
        let forwarded =
            exchange(&mut channel, &["myapp://x".to_owned()]).expect("hand-off acknowledged");
        assert_eq!(forwarded, 1);
        assert_eq!(channel.outgoing, encode_request(&["myapp://x".to_owned()]));

        // A wrong acknowledgement is a protocol violation.
        let mut channel = Channel {
            incoming: std::io::Cursor::new(b"NOPE\n".to_vec()),
            outgoing: Vec::new(),
        };
        assert!(matches!(
            exchange(&mut channel, &[]),
            Err(Error::Platform(_))
        ));

        // An empty reply (foreign peer / closed connection) fails loudly.
        let mut channel = Channel {
            incoming: std::io::Cursor::new(Vec::new()),
            outgoing: Vec::new(),
        };
        assert!(matches!(
            exchange(&mut channel, &[]),
            Err(Error::Platform(_))
        ));
    }

    #[test]
    fn parsing_rejects_foreign_and_malformed_headers() {
        assert_eq!(parse_header("NEXUM/1 2\n"), Some(2));
        assert_eq!(parse_header("HTTP/1.1 200\n"), None);
        assert_eq!(parse_header("NEXUM/1 not-a-number\n"), None);
        assert_eq!(parse_header("NEXUM/1\n"), None);
        assert_eq!(parse_header(""), None);
    }

    #[cfg(unix)]
    #[test]
    fn runtime_directory_prefers_xdg_and_falls_back() {
        let fallback = Path::new("/tmp");
        assert_eq!(
            runtime_directory_from(Some("/run/user/1000".to_owned()), fallback),
            PathBuf::from("/run/user/1000")
        );
        // An empty variable counts as unset.
        assert_eq!(
            runtime_directory_from(Some(String::new()), fallback),
            PathBuf::from("/tmp")
        );
        assert_eq!(
            runtime_directory_from(None, fallback),
            PathBuf::from("/tmp")
        );
    }
}
