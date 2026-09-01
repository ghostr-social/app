mod rescue_feedback;

pub use rescue_feedback::TransportRescueFeedback;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportRescueReason {
    EtaUnavailable,
    EtaTooLong,
    DeliveryFailed,
    GraceExpired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportRescue {
    pub reason: TransportRescueReason,
    pub rank_displacement: u32,
    pub wait_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryPlayback {
    pub session: PlaybackSession,
    pub sequence: PlaybackObservationSequence,
    pub observation: PlaybackObservation,
}
use ghostr_engine::playback::{PlaybackObservation, PlaybackObservationSequence, PlaybackSession};
