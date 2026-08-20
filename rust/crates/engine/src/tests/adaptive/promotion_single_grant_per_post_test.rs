use crate::adaptive::{AdaptivePlayabilityPolicy, PlayableRange, RetrievalRequest};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

const TOTAL: u64 = 800_000;

#[test]
fn one_post_has_at_most_one_contingent_whole_response_owner() {
    let mut input = snapshot(2, 40_000_000, 30_000, 2);
    let candidate = &mut input.candidates[1];
    let post = candidate.post.clone();
    candidate.total_bytes = Some(TOTAL);
    candidate.playable_ranges = (0..4)
        .map(|index| PlayableRange {
            bytes: ByteRange::new(index * 200_000, (index + 1) * 200_000),
            playable_ms: 2_000,
        })
        .collect();

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let work: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == post)
        .collect();
    let grants = work
        .iter()
        .filter(|work| {
            matches!(
                work.request,
                RetrievalRequest::FetchRange {
                    promotion: Some(_),
                    ..
                }
            )
        })
        .count();

    assert_eq!(grants, 1, "{plan:#?}");
    assert_eq!(work.len(), 1, "siblings must wait for response semantics");
}
