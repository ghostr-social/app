use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::tests::provisional_handoff_fixture::{handoff_to_hls_state, CURRENT};
use crate::tests::provisional_handoff_plan_fixture::{
    generated_cancels, plan_hls, plan_hls_with_retry,
};
use ghostr_engine::adaptive::{
    FeedOffset, HlsBootstrapStage, HlsBootstrapState, HlsCandidateSnapshot, HlsObjectCursor,
    HlsTransport, ViewProbability,
};
use ghostr_engine::{ActionId, PostId};
use std::collections::HashSet;

#[test]
fn pending_hls_current_reserves_a_slot_before_retaining_futures() {
    let (state, active) = handoff_to_hls_state();

    let work = plan_hls(state, &active, &[pending_current(Default::default())]);

    assert_eq!(generated_cancels(&work), HashSet::from([ActionId::new(1)]));
    assert!(work.retained.contains(&ActionId::new(2)));
    assert!(!work.retained.contains(&ActionId::new(1)));
}

#[test]
fn continued_live_response_does_not_reserve_another_request() {
    let (state, active) = handoff_to_hls_state();
    let cursor = HlsObjectCursor::new(
        1,
        0,
        None,
        HlsTransport::ContinueLive {
            response: ActionId::new(9),
        },
    );

    let work = plan_hls(state, &active, &[pending_current(cursor)]);

    assert!(generated_cancels(&work).is_empty());
    assert!(work.retained.contains(&ActionId::new(1)));
    assert!(work.retained.contains(&ActionId::new(2)));
}

#[test]
fn cooling_hls_current_does_not_evict_a_future_for_unavailable_work() {
    let (state, active) = handoff_to_hls_state();
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry
        .cool_down_hls_until(PostId::new(CURRENT), 5_000)
        .expect("cooldown");

    let work = plan_hls_with_retry(
        state,
        &active,
        &[pending_current(Default::default())],
        &retry,
    );

    assert!(generated_cancels(&work).is_empty());
    assert!(work.retained.contains(&ActionId::new(1)));
    assert!(work.retained.contains(&ActionId::new(2)));
}

fn pending_current(cursor: HlsObjectCursor) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new(CURRENT),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid probability"),
        startup_value_ms: 2_000,
        cursor,
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: "https://media.example/root.m3u8".into(),
        },
        player_preparation: Default::default(),
    }
}
