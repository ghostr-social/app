use crate::adaptive::{AdaptivePlayabilityPolicy, MediaLayout, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::tests::support::set_reliable_total_bytes;
use crate::{ByteRange, PostId};

#[test]
fn whole_fallback_makes_room_before_it_can_be_admitted() {
    let mut input = snapshot(2, 20_000_000, 0, 2);
    input.storage = StorageSnapshot::new(1_000_000, 1_000_000);
    input.candidates[0].layout = MediaLayout::RequiresCompleteFile;
    set_reliable_total_bytes(&mut input.candidates[0], 300_000, 10_000);
    input.candidates[1].present = vec![ByteRange::new(0, 1_000_000)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let current = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p0"));
    assert!(
        current.is_some(),
        "whole fallback must not wait for its own admission: {plan:#?}"
    );
    assert_eq!(
        current
            .expect("valid test fixture")
            .request
            .reserved_network_bytes(),
        300_000
    );
    let released: u64 = plan.evictions.iter().map(|item| item.range.len()).sum();
    assert!(
        released >= 300_000,
        "whole response fits released storage: {plan:#?}"
    );
    assert!(plan
        .evictions
        .iter()
        .all(|item| item.post != PostId::new("p0")));
}

#[test]
fn an_oversized_whole_fallback_does_not_discard_other_media_for_an_impossible_admission() {
    let mut input = snapshot(2, 20_000_000, 0, 2);
    input.storage = StorageSnapshot::new(1_000_000, 900_000);
    input.candidates[0].layout = MediaLayout::RequiresCompleteFile;
    set_reliable_total_bytes(&mut input.candidates[0], 2_000_000, 10_000);
    input.candidates[1].present = vec![ByteRange::new(0, 900_000)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    assert!(
        plan.evictions.is_empty(),
        "no feasible current request: {plan:#?}"
    );
    assert!(plan
        .allocations
        .iter()
        .all(|work| work.post != PostId::new("p0")));
}

#[test]
fn range_bootstrap_uses_partial_room_without_reserving_an_optional_whole_promotion() {
    let mut input = snapshot(2, 20_000_000, 0, 2);
    input.storage = StorageSnapshot::new(1_000_000, 1_000_000);
    input.candidates[0].layout = MediaLayout::Unknown;
    set_reliable_total_bytes(&mut input.candidates[0], 1_000_000, 10_000);
    input.candidates[1].present = vec![ByteRange::new(0, 1_000_000)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let released: u64 = plan.evictions.iter().map(|item| item.range.len()).sum();
    assert_eq!(
        released, 10_000,
        "range bootstrap can use the soft headroom"
    );
    assert!(plan
        .allocations
        .iter()
        .any(|work| work.post == PostId::new("p0")));
}
