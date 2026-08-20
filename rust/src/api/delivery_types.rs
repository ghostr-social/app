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

/// What the native cache has proved for one exact playback asset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiPlaybackPreparationReadiness {
    Preparing,
    StructuralStartable,
}

/// One exact loopback asset selected for current or adjacent-next use.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlaybackPreparationAsset {
    pub delivery_id: String,
    pub representation_id: String,
    pub asset_id: String,
    pub playback_url: String,
    pub readiness: FfiPlaybackPreparationReadiness,
}

/// Atomic two-player preparation window from one manager plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlaybackPreparationPlan {
    pub revision: u64,
    /// The focused delivery even while no exact progressive asset exists.
    pub current_delivery_id: Option<String>,
    pub current: Option<FfiPlaybackPreparationAsset>,
    pub next: Option<FfiPlaybackPreparationAsset>,
}

/// Native-player evidence for one exact progressive playback attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiPlayerPreparationState {
    Initializing,
    Initialized,
    FirstFrameRendered,
    Failed,
    Released,
}

/// Authority and monotonic ordering for a player-preparation update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlayerPreparationReport {
    pub post_id: String,
    pub representation_id: String,
    pub asset_id: String,
    pub player_capability_generation: u64,
    pub client_epoch: u64,
    pub attempt_generation: u64,
    pub sequence: u64,
    pub state: FfiPlayerPreparationState,
    pub failure_kind: Option<String>,
    pub observed_monotonic_us: u64,
}
