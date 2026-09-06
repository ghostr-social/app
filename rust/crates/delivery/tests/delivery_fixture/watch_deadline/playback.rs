use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;

pub fn progress(generation: u64, sequence: u64, position_secs: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("current"), generation),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::from_secs(position_secs),
            Duration::from_secs(8),
            1_000,
            PlaybackPhase::Playing,
        )
        .expect("valid playback progress"),
    }
}
