use crate::manager::time::unix_time_ms;
use crate::tests::response_driven_promotion_fixture::SOURCE;
use crate::tests::response_driven_promotion_manager_support::fixture;
use crate::tests::response_fit_reservation_manager_test::covered_response;
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::ByteRange;

#[tokio::test]
async fn internal_first_covered_response_publishes_the_same_whole_state() {
    let mut fixture = fixture().await;
    let valid_until_ms = unix_time_ms().saturating_add(10_000);
    let (attempt, action) = fixture
        .worker
        .register_covered_response_for_test(&fixture.post, SOURCE, valid_until_ms)
        .await;
    fixture
        .worker
        .queue_response_for_test(attempt.clone(), covered_response());

    assert!(fixture.step().await);
    let actions = fixture.worker.active_actions_for_test();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].action_id(), attempt.id());
    assert_eq!(actions[0].effective_bytes(), ByteRange::new(0, 8));
    assert_eq!(actions[0].request(), planned_completion());
    assert!(!fixture.handle.decision_history_json().expect("history").contains("\"command\":\"promote\""));

    fixture.worker.finish_response_attempt_for_test(&attempt);
    fixture.store.release_action(&action).await;
    drop(fixture.store);
    tokio::fs::remove_dir_all(fixture.root).await.expect("fixture cleanup");
}

fn planned_completion() -> RetrievalRequest {
    RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact { expected_bytes: 8 },
        reason: WholeFetchReason::PlannedCompletion,
    }
}
