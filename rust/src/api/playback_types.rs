/// Explicit player phases crossing the Flutter/Rust boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiPlaybackPhase {
    Starting,
    Playing,
    NetworkStalled,
    Paused,
    Ended,
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
