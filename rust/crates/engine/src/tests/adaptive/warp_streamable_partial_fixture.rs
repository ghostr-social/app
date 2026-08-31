use crate::adaptive::{PlayabilitySnapshot, PlayableRange};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::tests::adaptive_support::snapshot;
use crate::tests::support::set_reliable_total_bytes;
use crate::ByteRange;

pub(super) const TOTAL: u64 = 293_999;

pub(super) fn partial_state(total: u64) -> PlayabilitySnapshot {
    let mut state = snapshot(2, 20_000_000, 20_000, 20);
    state.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    let candidate = &mut state.candidates[1];
    set_reliable_total_bytes(candidate, total, state.observed_at_ms);
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, total),
        playable_ms: 6_000,
    }];
    candidate.startup = StartupFootprint::new(
        vec![ByteRange::new(0, 65_536)],
        1_000,
        StartupProvenance::ClassicMp4V1,
    );
    candidate.present = vec![ByteRange::new(0, 65_536)];
    state
}
