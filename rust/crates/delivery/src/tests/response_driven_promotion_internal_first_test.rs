use crate::chunk::downloader::ResponseAdmission;
use crate::manager::time::unix_time_ms;
use crate::tests::response_driven_promotion_fixture::{promoted_request, response, SOURCE};
use crate::tests::response_driven_promotion_manager_support::{fixture, join};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};

#[tokio::test]
async fn internal_observation_before_authorization_is_idempotently_promoted() {
    let mut fixture = fixture().await;
    let valid_until_ms = unix_time_ms().saturating_add(10_000);
    let (attempt, action) = fixture
        .worker
        .register_promotable_response_for_test(&fixture.post, SOURCE, valid_until_ms)
        .await;
    fixture
        .worker
        .queue_response_for_test(attempt.clone(), response());

    assert!(fixture.step().await);
    let staged = fixture.worker.active_actions_for_test();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].action_id(), attempt.id());
    assert_eq!(staged[0].request(), capped_request());

    let admission =
        fixture
            .worker
            .authorize_response_for_test(attempt.clone(), action.clone(), response());
    fixture.worker.wait_for_response_request_for_test().await;
    assert!(fixture.step().await);
    assert_eq!(join(admission).await, ResponseAdmission::Proceed);
    let opened = fixture.worker.active_actions_for_test();
    assert_eq!(opened.len(), 1);
    assert_eq!(opened[0].action_id(), attempt.id());
    assert_eq!(opened[0].request(), promoted_request());

    fixture.worker.finish_response_attempt_for_test(&attempt);
    fixture.store.release_action(&action).await;
    drop(fixture.store);
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
}

fn capped_request() -> RetrievalRequest {
    RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped { maximum_bytes: 8 },
        reason: WholeFetchReason::PromotedResponse,
    }
}
