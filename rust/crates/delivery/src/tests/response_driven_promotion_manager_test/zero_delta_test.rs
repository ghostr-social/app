use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseAdmission, ResponseObservation, ResponseWriteMode,
};
use crate::manager::time::unix_time_ms;
use crate::tests::response_driven_promotion_fixture::SOURCE;
use crate::tests::response_driven_promotion_manager_support::{fixture, join};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::ByteRange;

#[tokio::test]
async fn nonzero_range_can_promote_semantics_without_more_authority() {
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
    let active = fixture.worker.active_actions_for_test();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].effective_bytes(), ByteRange::new(0, 4));
    assert_eq!(active[0].reserved_storage_bytes(), 4);
    assert_eq!(active[0].request(), promoted_request());
    assert_zero_authority_trace(&fixture.handle.decision_history_json().expect("history"));
    let finished = fixture.worker.finish_response_attempt_for_test(&attempt);
    assert_eq!(
        finished
            .network_reservation()
            .expect("reservation")
            .committed_bytes(),
        4
    );
    fixture.store.release_action(&action).await;
    drop(fixture.store);
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("fixture cleanup");
}

fn response() -> OpenedResponse {
    let contract = WholeBodyContract::Exact { expected_bytes: 4 };
    OpenedResponse::new(
        ResponseObservation::Body {
            request: promoted_request(),
            total: Some(4),
            range_support: Some(false),
            promoted: true,
        },
        None,
        ResponseWriteMode::SingleResponse(contract),
        HttpResponseEvidence {
            final_url: SOURCE.into(),
            status: 200,
            content_type: Some("video/mp4".into()),
            validator: None,
            observed: 0.into(),
        },
    )
}

fn promoted_request() -> RetrievalRequest {
    RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact { expected_bytes: 4 },
        reason: WholeFetchReason::PromotedResponse,
    }
}

fn assert_zero_authority_trace(json: &str) {
    let evidence: serde_json::Value = serde_json::from_str(json).expect("valid JSON");
    let selected = evidence["decisions"]["records"]
        .as_array()
        .expect("records")
        .iter()
        .filter_map(|item| item["warp_decision"]["selected"].as_object())
        .find(|item| item["command"]["command"] == "promote")
        .expect("promotion");
    assert_eq!(selected["resources"]["network_bytes"], 4);
    assert_eq!(selected["authorized_resources"]["network_bytes"], 0);
}
