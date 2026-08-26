use ghostr_delivery::playback_admission::{PlaybackAdmissionSnapshot, PlaybackRejection};

/// Explicit player phases crossing the Flutter/Rust boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiPlaybackPhase {
    Starting,
    Playing,
    NetworkStalled,
    Paused,
    Ended,
    Failed,
    Inactive,
}

/// One self-contained, ordered playback sample.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlaybackObservation {
    pub post_id: String,
    pub generation: u64,
    pub sequence: u64,
    pub phase: FfiPlaybackPhase,
    pub position_ms: u64,
    pub buffered_extent_ms: u64,
    pub playback_rate_milli: u32,
}

/// One user-visible Flutter frame for an exact playback session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlaybackPresentation {
    pub(crate) post_id: String,
    pub(crate) generation: u64,
    pub(crate) sequence: u64,
    pub(crate) observed_at_ms: u64,
}

/// Delivery manager decisions and latest accepted identity in this process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiPlaybackAdmissionSnapshot {
    pub(crate) accepted: u64,
    pub(crate) inactive_delivery: u64,
    pub(crate) stale_session: u64,
    pub(crate) stale_sequence: u64,
    pub(crate) last_accepted_delivery_id: Option<String>,
}

impl From<PlaybackAdmissionSnapshot> for FfiPlaybackAdmissionSnapshot {
    fn from(snapshot: PlaybackAdmissionSnapshot) -> Self {
        let last_accepted_delivery_id = snapshot
            .last_accepted()
            .map(|post| post.as_str().to_owned());
        let counters = snapshot.counters();
        Self {
            accepted: counters.accepted(),
            inactive_delivery: counters.rejected(PlaybackRejection::InactiveDelivery),
            stale_session: counters.rejected(PlaybackRejection::StaleSession),
            stale_sequence: counters.rejected(PlaybackRejection::StaleSequence),
            last_accepted_delivery_id,
        }
    }
}
