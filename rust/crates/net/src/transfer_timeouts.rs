//! Transfer-phase timeouts for media probes and chunk downloads,
//! consistent with the Dart-side video download timeouts: ~15 s to
//! response headers and ~15 s maximum gap between body chunks. The
//! outbound client's connect and whole-request timeouts still apply
//! underneath these.

use std::time::Duration;

/// Longest wait for response headers after sending a request.
pub const HEADERS_TIMEOUT: Duration = Duration::from_secs(15);

/// Longest tolerated silence between two body chunks.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(15);

/// Timeout pair applied by the probe and the chunk downloader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferTimeouts {
    pub headers: Duration,
    pub idle: Duration,
}

impl TransferTimeouts {
    pub fn new(headers: Duration, idle: Duration) -> Self {
        Self { headers, idle }
    }
}

impl Default for TransferTimeouts {
    fn default() -> Self {
        Self::new(HEADERS_TIMEOUT, IDLE_TIMEOUT)
    }
}
