use crate::adaptive::{AdaptivePlayabilityPolicy, MediaLayout, PlayableRange};
use crate::tests::adaptive_support::{frontier, snapshot};
use crate::{ByteRange, PostId};

#[test]
fn complete_file_media_waits_until_playable_coverage_can_pay_its_delivery_time() {
    let policy = AdaptivePlayabilityPolicy;
    let mut unsafe_input = snapshot(3, 40_000_000, 5_000, 2);
    make_complete_file(&mut unsafe_input.candidates[1]);
    let mut safe_input = unsafe_input.clone();
    safe_input.playback.buffer_ahead_ms = 30_000;

    let deferred = policy.plan(&unsafe_input);
    let admitted = policy.plan(&safe_input);

    assert!(
        !frontier(&deferred).contains(&PostId::new("p1")),
        "{deferred:#?}"
    );
    assert!(
        frontier(&deferred).contains(&PostId::new("p2")),
        "{deferred:#?}"
    );
    assert!(
        frontier(&admitted).contains(&PostId::new("p1")),
        "{admitted:#?}"
    );
}

fn make_complete_file(candidate: &mut crate::adaptive::CandidateSnapshot) {
    candidate.layout = MediaLayout::RequiresCompleteFile;
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, 20_000_000),
        playable_ms: 60_000,
    }];
}
