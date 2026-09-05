use crate::playback::{
    AdaptiveBufferPolicy, ContinuationConditions, EstimateConfidence, NetworkConditions,
    PlaybackObservation, PlaybackPhase,
};
use crate::tests::media_timeline_dependency_support::tail_timeline;
use crate::tests::media_timeline_support::classic_moov;
use crate::ByteRange;
use core::time::Duration;

#[test]
fn buffer_target_uses_completed_sample_dependencies_instead_of_mean_bitrate() {
    let offsets: Vec<u32> = (1..=24).map(|sample| sample * 100).collect();
    let movie = classic_moov(&offsets, &[100; 24]);
    let timeline = tail_timeline(&movie);
    let present = [
        ByteRange::new(0, 100),
        ByteRange::new(2_500, 10_000 + movie.len() as u64),
    ];
    let conditions = ContinuationConditions {
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::ZERO,
            1_000,
            PlaybackPhase::Playing,
        )
        .expect("fixture"),
        // Half the shared path is reserved for the other protected item.
        network: NetworkConditions::new(125, 0, Duration::ZERO, EstimateConfidence::High),
    };
    let target = AdaptiveBufferPolicy::default()
        .target_for_timeline(&timeline, conditions, &present)
        .expect("fixture");
    assert_eq!(target.required(), Duration::from_millis(8_450));
    let cached = [ByteRange::new(0, 10_000 + movie.len() as u64)];
    let target = AdaptiveBufferPolicy::default()
        .target_for_timeline(&timeline, conditions, &cached)
        .expect("fixture");
    assert_eq!(target.required(), Duration::from_secs(4));
}
