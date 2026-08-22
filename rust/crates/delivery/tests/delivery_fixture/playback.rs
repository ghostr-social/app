//! Playback evidence builders shared by delivery integration scenarios.

use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;
use std::time::Duration;

pub fn playing(post: &str, buffer: Duration) -> DeliveryPlayback {
    playing_at(post, buffer, 1)
}

pub fn playing_at(post: &str, buffer: Duration, sequence: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new(post), 1),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            buffer,
            1_000,
            PlaybackPhase::Playing,
        )
        .unwrap(),
    }
}
