use crate::manager::time::unix_time_ms;
use crate::tests::response_driven_promotion_fixture::{response, SOURCE};
use crate::tests::response_driven_promotion_manager_support::fixture;

#[tokio::test]
async fn manager_rejection_fences_the_paused_response_before_another_plan() {
    let mut fixture = fixture().await;
    let valid_until_ms = unix_time_ms().saturating_add(10_000);
    let (attempt, action) = fixture
        .worker
        .register_promotable_response_for_test(&fixture.post, SOURCE, valid_until_ms)
        .await;

    assert!(
        !fixture
            .worker
            .reject_unselected_response_for_test(&attempt, &action, &response())
            .await
    );
    let actions = fixture.worker.active_actions_for_test();
    assert_eq!(actions.len(), 1);
    assert!(actions[0].cancelling());

    fixture.worker.finish_response_attempt_for_test(&attempt);
    fixture.store.release_action(&action).await;
    drop(fixture.store);
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
}
