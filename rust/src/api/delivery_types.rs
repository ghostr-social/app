//! Data shapes of the versioned FFI delivery contract.

/// How the engine delivers one playable media item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiMediaDelivery {
    Progressive,
    Hls,
}

/// One entry of the ordered focus window, including the current item.
/// `post_id` doubles as the partial-store key and the gateway `?id=`
/// value, so it must stay within `[A-Za-z0-9_-]`.
#[derive(Clone, Debug)]
pub struct FfiFocusItem {
    pub post_id: String,
    /// Playback candidates in preference order (imeta url + fallbacks).
    pub urls: Vec<String>,
    /// HLS items ride in the window for correct scroll distances but
    /// are never downloaded progressively.
    pub delivery: FfiMediaDelivery,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_ms: Option<u64>,
}

/// What a delivery event reports about one post.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiDeliveryEventKind {
    /// Startability changed (or the post is newly watched).
    Readiness,
    /// Bytes or the known total moved without a startability change.
    Progress,
    /// The store could not be read for this post; see `detail`.
    Error,
}

/// One per-post delivery update streamed to Dart.
#[derive(Clone, Debug)]
pub struct FfiDeliveryEvent {
    pub post_id: String,
    pub kind: FfiDeliveryEventKind,
    pub startable: bool,
    pub bytes_present: u64,
    pub total_bytes: Option<u64>,
    /// Best current completion estimate; absent when the engine cannot
    /// make a defensible estimate.
    pub eta_ms: Option<u64>,
    pub detail: Option<String>,
}
