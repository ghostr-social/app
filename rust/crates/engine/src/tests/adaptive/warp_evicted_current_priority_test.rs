use crate::adaptive::{
    AdaptivePlayabilityPolicy, CandidateSnapshot, MediaLayout, PlannerContext, PlayabilitySnapshot,
    PlayableRange, PlayerPreparation, StorageSnapshot, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::playback::PlaybackPhase;
use crate::tests::adaptive_support::snapshot;
use crate::tests::support::set_reliable_total_bytes;
use crate::ByteRange;

#[test]
fn evicted_current_reacquisition_precedes_faster_future_whole_downloads() {
    let input = cache_return();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let selected = decision
        .selected
        .expect("current has feasible missing bytes");
    assert_eq!(selected.node.post, input.playback.current);
}

fn cache_return() -> PlayabilitySnapshot {
    let mut input = snapshot(3, 2_500_000, 0, 0);
    input.playback.phase = PlaybackPhase::Starting;
    input.storage = StorageSnapshot::new(716_800, 0);
    for candidate in &mut input.candidates {
        configure_missing_media(candidate);
    }
    input.candidates[0].recently_evicted = vec![ByteRange::new(0, 293_999)];
    input.candidates[1].layout = MediaLayout::RequiresCompleteFile;
    input.candidates[2].layout = MediaLayout::RequiresCompleteFile;
    input
}

fn configure_missing_media(candidate: &mut CandidateSnapshot) {
    let bytes = if candidate.feed_offset.value() == 0 {
        293_999
    } else {
        200_000
    };
    set_reliable_total_bytes(candidate, bytes, 10_000);
    candidate.duration_ms = 6_000;
    candidate.startup = None;
    candidate.player_preparation = PlayerPreparation::Unverified;
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, bytes),
        playable_ms: 6_000,
    }];
}
