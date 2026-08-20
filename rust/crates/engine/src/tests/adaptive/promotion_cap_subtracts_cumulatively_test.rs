use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlayableRange, RetrievalRequest, StorageSnapshot,
};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

const TOTAL: u64 = 600_000;

#[test]
fn contingent_promotions_share_one_hard_storage_budget() {
    let mut input = snapshot(4, 40_000_000, 30_000, 60);
    input.storage = StorageSnapshot::new(1_000_000, 100_000);
    for candidate in &mut input.candidates[1..=2] {
        candidate.total_bytes = Some(TOTAL);
        candidate.playable_ranges = vec![PlayableRange {
            bytes: ByteRange::new(0, TOTAL),
            playable_ms: 8_000,
        }];
    }

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let reserved: u64 = plan
        .allocations
        .iter()
        .filter_map(|work| match work.request {
            RetrievalRequest::FetchRange {
                promotion: Some(grant),
                ..
            } => Some(grant.maximum_bytes),
            _ => None,
        })
        .sum();

    assert_eq!(reserved, TOTAL, "{plan:#?}");
    assert!(reserved <= input.storage.available_bytes());
}

#[test]
fn timeline_promotion_consumes_its_full_reservation_before_later_posts() {
    let mut input = snapshot(4, 40_000_000, 30_000, 60);
    input.storage = StorageSnapshot::new(1_000_000, 100_000);
    for candidate in &mut input.candidates[1..=2] {
        candidate.total_bytes = Some(TOTAL);
        candidate.playable_ranges = (0..3)
            .map(|index| PlayableRange {
                bytes: ByteRange::new(index * 200_000, (index + 1) * 200_000),
                playable_ms: 2_000,
            })
            .collect();
    }
    input.candidates[1].timeline_probe = input.candidates[1].playable_ranges.last().copied();

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let reserved: u64 = plan
        .allocations
        .iter()
        .map(|work| work.request.reserved_network_bytes())
        .sum();

    assert!(reserved <= input.storage.available_bytes(), "{plan:#?}");
}
