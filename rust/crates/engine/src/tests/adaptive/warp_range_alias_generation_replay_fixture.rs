use super::{RANGE_END, RANGE_START, TOTAL};
use crate::adaptive::{MediaLayout, PlayableRange};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::tests::adaptive_support::snapshot;
use crate::tests::support::set_reliable_total_bytes;
use crate::{ByteRange, PostId};

pub(super) fn partial_state() -> crate::adaptive::PlayabilitySnapshot {
    let mut state = snapshot(2, 20_000_000, 20_000, 20);
    state.request_slice_bytes = RANGE_END - RANGE_START;
    state.playback.current = PostId::new("p1");
    state.playback.buffer_ahead_ms = 0;
    state.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    let candidate = &mut state.candidates[1];
    candidate.layout = MediaLayout::Unknown;
    set_reliable_total_bytes(candidate, TOTAL, state.observed_at_ms);
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, TOTAL),
        playable_ms: 6_000,
    }];
    candidate.startup = StartupFootprint::new(
        vec![ByteRange::new(0, RANGE_START)],
        1_000,
        StartupProvenance::ClassicMp4V1,
    );
    candidate.present = vec![ByteRange::new(0, RANGE_START)];
    state
}
