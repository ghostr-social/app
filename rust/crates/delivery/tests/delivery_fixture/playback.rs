//! Playback evidence builders shared by delivery integration scenarios.

use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;
use std::time::Duration;

pub fn playing(post: &str, buffer: Duration) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new(post), 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            buffer,
            1_000,
            PlaybackPhase::Playing,
        )
        .unwrap(),
    }
}
