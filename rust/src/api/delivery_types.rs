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
    /// Inline-only preview payload; remote thumbnails are not readiness.
    pub blurhash: Option<String>,
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
    /// Playback is terminally blocked for the current delivery binding.
    Failed,
}

/// One per-post delivery update streamed to Dart.
#[derive(Clone, Debug)]
pub struct FfiDeliveryEvent {
    pub(crate) post_id: String,
    pub(crate) kind: FfiDeliveryEventKind,
    pub(crate) startable: bool,
    pub(crate) bytes_present: u64,
    pub(crate) total_bytes: Option<u64>,
    /// Best current completion estimate; absent when the engine cannot
    /// make a defensible estimate.
    pub(crate) eta_ms: Option<u64>,
    pub(crate) detail: Option<String>,
    /// Exact progressive representation when this event has one.
    pub(crate) representation_id: Option<String>,
    /// Exact progressive capability bound to the observed content revision.
    pub(crate) asset_id: Option<String>,
}

/// What the native cache has proved for one exact playback asset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiPlaybackPreparationReadiness {
    Preparing,
    StructuralStartable,
    Ready,
}

/// One exact loopback asset selected for current or adjacent-next use.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlaybackPreparationAsset {
    pub(crate) delivery_id: String,
    pub(crate) representation_id: String,
    pub(crate) source_representation_id: String,
    pub(crate) asset_id: String,
    pub(crate) playback_url: String,
    pub(crate) readiness: FfiPlaybackPreparationReadiness,
}

/// Atomic playback preparation window from one manager plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlaybackPreparationPlan {
    pub(crate) revision: u64,
    /// The focused delivery even while no exact progressive asset exists.
    pub(crate) current_delivery_id: Option<String>,
    pub(crate) current: Option<FfiPlaybackPreparationAsset>,
    /// Every certified upcoming asset, in feed order.
    pub(crate) upcoming: Vec<FfiPlaybackPreparationAsset>,
    /// Compatibility projection of the first upcoming asset.
    pub(crate) next: Option<FfiPlaybackPreparationAsset>,
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

/// Actor-applied result for one immutable player-preparation report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiPlayerPreparationDisposition {
    Applied,
    Duplicate,
    Stale,
    MissingInitial,
    Rejected,
    Saturated,
    Unavailable,
    Closed,
    NotAdmitted,
}

/// Authority and monotonic ordering for a player-preparation update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlayerPreparationReport {
    pub(crate) post_id: String,
    pub(crate) representation_id: String,
    pub(crate) asset_id: String,
    pub(crate) player_capability_generation: u64,
    pub(crate) client_epoch: u64,
    pub(crate) attempt_generation: u64,
    pub(crate) sequence: u64,
    pub(crate) state: FfiPlayerPreparationState,
    pub(crate) failure_kind: Option<String>,
    pub(crate) observed_monotonic_us: u64,
}
