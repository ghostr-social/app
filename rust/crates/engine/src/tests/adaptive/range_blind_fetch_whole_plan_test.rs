use crate::adaptive::{
    AdaptivePlayabilityPolicy, MediaLayout, PlayableRange, RetrievalRequest, StorageSnapshot,
    WholeBodyContract, WholeFetchReason,
};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

const TOTAL: u64 = 800_000;

#[test]
fn complete_file_with_a_cached_prefix_uses_one_full_get_and_reserves_the_whole_body() {
    let mut input = snapshot(2, 40_000_000, 30_000, 2);
    make_complete(&mut input.candidates[1]);

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let work: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == PostId::new("p1"))
        .collect();

    assert_eq!(work.len(), 1, "{plan:#?}");
    assert_eq!(
        work[0].request,
        RetrievalRequest::FetchWhole {
            contract: WholeBodyContract::Capped {
                maximum_bytes: TOTAL,
            },
            reason: WholeFetchReason::PlannedCompletion,
        }
    );
    assert_eq!(work[0].request.reserved_network_bytes(), TOTAL);
}

#[test]
fn complete_file_is_deferred_when_side_by_side_storage_cannot_hold_the_full_response() {
    let mut input = snapshot(2, 40_000_000, 30_000, 2);
    make_complete(&mut input.candidates[1]);
    input.storage = StorageSnapshot::new(1_000_000, 200_001);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        plan.allocations
            .iter()
            .all(|work| work.post != PostId::new("p1")),
        "{plan:#?}"
    );
}

fn make_complete(candidate: &mut crate::adaptive::CandidateSnapshot) {
    candidate.total_bytes = Some(TOTAL);
    candidate.layout = MediaLayout::RequiresCompleteFile;
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, TOTAL),
        playable_ms: 8_000,
    }];
    candidate.present = vec![ByteRange::new(0, 64 * 1024)];
}
