use crate::chunk::downloader::ResponseAdmission;
use crate::manager::time::unix_time_ms;
use crate::tests::response_driven_promotion_fixture::{
    assert_promotion_trace, promoted_request, response, SOURCE,
};
use crate::tests::response_driven_promotion_manager_support::{fixture, join};
use ghostr_engine::ByteRange;

#[path = "response_driven_promotion_manager_test/zero_delta_test.rs"]
mod zero_delta_test;

#[tokio::test]
async fn observed_200_promotes_and_authorizes_the_same_response_in_one_step() {
    let mut fixture = fixture().await;
    let valid_until_ms = unix_time_ms().saturating_add(10_000);
    let (attempt, action) = fixture
        .worker
        .register_promotable_response_for_test(&fixture.post, SOURCE, valid_until_ms)
        .await;
    let admission =
        fixture
            .worker
            .authorize_response_for_test(attempt.clone(), action.clone(), response());
    fixture.worker.wait_for_response_request_for_test().await;

    assert!(fixture.step().await);
    assert_eq!(join(admission).await, ResponseAdmission::Proceed);
    let actions = fixture.worker.active_actions_for_test();
    assert_eq!(actions.len(), 1, "{actions:#?}");
    assert_eq!(actions[0].action_id(), attempt.id());
    assert_eq!(actions[0].effective_bytes(), ByteRange::new(0, 8));
    assert_eq!(actions[0].reserved_storage_bytes(), 8);
    assert_eq!(actions[0].request(), promoted_request());
    assert_promotion_trace(&fixture.handle, attempt.id().value(), valid_until_ms);
    let finished = fixture.worker.finish_response_attempt_for_test(&attempt);
    assert_eq!(
        finished
            .network_reservation()
            .expect("committed reservation")
            .committed_bytes(),
        8
    );
    fixture.store.release_action(&action).await;
    drop(fixture.store);
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
}
