use crate::manager::state::DeliveryState;
use ghostr_engine::{ByteRange, DataUsageLevel, EngineParams, PostId};

#[test]
fn policy_eviction_history_normalizes_exact_ranges_and_clears_with_state() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let post = PostId::new("evicted");
    state.record_policy_evictions(post.clone(), &[10..20, 15..30]);

    assert_eq!(state.recently_evicted(&post), vec![ByteRange::new(10, 30)]);

    state.clear();
    assert!(state.recently_evicted(&post).is_empty());
}
