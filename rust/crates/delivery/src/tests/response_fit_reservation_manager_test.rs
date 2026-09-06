use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseAdmission, ResponseObservation, ResponseWriteMode,
};
use crate::manager::time::unix_time_ms;
use crate::tests::response_driven_promotion_fixture::SOURCE;
use crate::tests::response_driven_promotion_manager_support::{fixture, join};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::ByteRange;

#[tokio::test]
async fn a_whole_response_covered_by_the_prefix_opens_without_promotion() {
    let mut fixture = fixture().await;
    let valid_until_ms = unix_time_ms().saturating_add(10_000);
    let (attempt, action) = fixture
        .worker
        .register_covered_response_for_test(&fixture.post, SOURCE, valid_until_ms)
        .await;
    let admission = fixture.worker.authorize_response_for_test(
        attempt.clone(),
        action.clone(),
        covered_response(),
    );
    fixture.worker.wait_for_response_request_for_test().await;

    assert!(fixture.step().await);
    assert_eq!(join(admission).await, ResponseAdmission::Proceed);
    let actions = fixture.worker.active_actions_for_test();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].action_id(), attempt.id());
    assert_eq!(actions[0].effective_bytes(), ByteRange::new(0, 8));
    assert_eq!(actions[0].reserved_storage_bytes(), 8);
    assert!(!fixture
        .handle
        .decision_history_json()
        .expect("history")
        .contains("\"command\":\"promote\""));
    let finished = fixture.worker.finish_response_attempt_for_test(&attempt);
    assert_eq!(
        finished
            .network_reservation()
            .expect("reservation")
            .committed_bytes(),
        8
    );
    fixture.store.release_action(&action).await;
    drop(fixture.store);
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("fixture cleanup");
}

pub(super) fn covered_response() -> OpenedResponse {
    let contract = WholeBodyContract::Exact { expected_bytes: 8 };
    OpenedResponse::new(
        ResponseObservation::Body {
            request: RetrievalRequest::FetchWhole {
                contract,
                reason: WholeFetchReason::PlannedCompletion,
            },
            total: Some(8),
            range_support: Some(false),
            promoted: false,
        },
        None,
        ResponseWriteMode::SingleResponse(contract),
        HttpResponseEvidence {
            request_selection: None,
            final_url: SOURCE.into(),
            status: 200,
            content_type: Some("video/mp4".into()),
            validator: None,
            observed: 0.into(),
        },
    )
}
