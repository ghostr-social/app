use crate::api::delivery::playback_mapping::playback_update;
use crate::api::playback_types::{FfiPlaybackObservation, FfiPlaybackPhase};
use crate::engine::playback::PlaybackPhase;
use std::time::Duration;

#[test]
fn maps_explicit_playback_evidence_into_the_delivery_domain() {
    let mapped = playback_update(FfiPlaybackObservation {
        post_id: "video_1".into(),
        generation: 7,
        sequence: 12,
        phase: FfiPlaybackPhase::NetworkStalled,
        position_ms: 2_500,
        buffered_extent_ms: 4_000,
        playback_rate_milli: 1_250,
    })
    .expect("valid playback update");

    assert_eq!(mapped.session.post().as_str(), "video_1");
    assert_eq!(mapped.session.generation(), 7);
    assert_eq!(mapped.observation.phase(), PlaybackPhase::NetworkStalled);
    assert_eq!(
        mapped.observation.buffer_ahead(),
        Duration::from_millis(1_500)
    );
    assert_eq!(mapped.observation.playback_rate_milli(), 1_250);
}

#[test]
fn maps_every_explicit_player_phase_without_boolean_inference() {
    let phases = [
        (FfiPlaybackPhase::Starting, PlaybackPhase::Starting),
        (FfiPlaybackPhase::Playing, PlaybackPhase::Playing),
        (FfiPlaybackPhase::Paused, PlaybackPhase::Paused),
        (FfiPlaybackPhase::Ended, PlaybackPhase::Ended),
        (FfiPlaybackPhase::Inactive, PlaybackPhase::Inactive),
    ];

    for (input, expected) in phases {
        assert_eq!(PlaybackPhase::from(input), expected);
    }
}
