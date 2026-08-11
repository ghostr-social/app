use crate::api::delivery::playback_mapping::playback_update;
use crate::api::playback_types::{FfiPlaybackObservation, FfiPlaybackPhase};

#[test]
fn rejects_invalid_ids_timelines_and_playback_rates_at_the_ffi_edge() {
    assert!(playback_update(input("bad/id", 0, 1_000)).is_err());
    assert!(playback_update(input("video", 2_000, 1_000)).is_err());
    assert!(playback_update(input("video", 0, 0)).is_err());
    assert!(playback_update(input("video", 0, u32::from(u16::MAX) + 1)).is_err());
    let mut missing_generation = input("video", 0, 1_000);
    missing_generation.generation = 0;
    assert!(playback_update(missing_generation).is_err());
    let mut missing_sequence = input("video", 0, 1_000);
    missing_sequence.sequence = 0;
    assert!(playback_update(missing_sequence).is_err());
}

fn input(post_id: &str, position_ms: u64, rate: u32) -> FfiPlaybackObservation {
    FfiPlaybackObservation {
        post_id: post_id.into(),
        generation: 1,
        sequence: 1,
        phase: FfiPlaybackPhase::Playing,
        position_ms,
        buffered_extent_ms: 1_000,
        playback_rate_milli: rate,
    }
}
