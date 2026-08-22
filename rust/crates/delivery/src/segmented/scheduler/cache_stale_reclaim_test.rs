use super::cache_pressure_reclaim_test::{focus, store, MIB};
use super::SegmentedDelivery;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::PostId;

#[test]
fn stale_reservation_cannot_reclaim_a_ready_handoff() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus(1, 0));
    delivery.pending.clear();
    store(&cache, "first", 1, &[MIB, MIB, 8 * MIB, 8 * MIB]);
    assert!(cache.mark_stage_ready(&PostId::new("first"), 1));
    delivery.apply_focus(&focus(2, 1));
    store(&cache, "second", 2, &[MIB, MIB, 8 * MIB]);

    assert!(!cache.mark_stage_preparing(
        &PostId::new("second"),
        3,
        500,
        HlsBootstrapStage::FirstSegment.maximum_bytes(),
    ));
    assert_eq!(cache.snapshot("first").phase, SegmentedPhase::Ready);
    assert!(cache.object("https://first.example/root.m3u8").is_some());
    assert_eq!(cache.physical_available_bytes(), 4 * MIB as u64);
}
