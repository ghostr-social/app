//! Data shapes of FFI contract v1 (plan §2). Additive changes only
//! until phase 2 — the surface is frozen.

/// One entry of the ordered focus window, including the current item.
/// `post_id` doubles as the partial-store key and the gateway `?id=`
/// value, so it must stay within `[A-Za-z0-9_-]`.
#[derive(Clone, Debug)]
pub struct FfiFocusItem {
    pub post_id: String,
    /// Playback candidates in preference order (imeta url + fallbacks).
    pub urls: Vec<String>,
    /// `"progressive"` or `"hls"`. HLS items ride in the window for
    /// correct scroll distances but are never downloaded progressively.
    pub delivery: String,
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

/// One per-post delivery update streamed to Dart (plan §2 row 5).
#[derive(Clone, Debug)]
pub struct FfiDeliveryEvent {
    pub post_id: String,
    pub kind: FfiDeliveryEventKind,
    pub startable: bool,
    pub bytes_present: u64,
    pub total_bytes: Option<u64>,
    pub detail: Option<String>,
}
