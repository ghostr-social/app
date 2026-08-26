use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseRejection, ResponseWriteMode,
};
use crate::tests::response_evidence_fixture::{
    etag, wait_for_validator, EVENT_TIMEOUT, SOURCE,
};
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::evidence::EvidenceTime;

#[tokio::test]
async fn malformed_new_generation_revokes_cached_bytes_and_validator() {
    let (parser, mut started) = GatedTimelineParser::new(None, 1);
    let mut fixture = TimelineManagerFixture::new(std::sync::Arc::<GatedTimelineParser>::clone(&parser)).await;
    fixture.focus();
    assert!(fixture.step().await);
    assert_eq!(started.recv().await, Some(0));
    parser.release(0);
    let attempt = fixture
        .worker
        .register_response_attempt_for_test(&fixture.post, SOURCE);
    fixture.worker.queue_response_for_test(
        attempt.clone(),
        response(
            ResponseObservation::Ignored {
                total: None,
                range_support: Some(false),
            },
            "v1",
            1,
        ),
    );
    wait_for_validator(&mut fixture, etag("v1")).await;
    fixture
        .store
        .write_range(fixture.post.as_str(), 0, &[7; 4])
        .await
        .expect("valid test fixture");

    fixture.worker.queue_response_for_test(
        attempt,
        response(
            ResponseObservation::Rejected(ResponseRejection::Semantics),
            "v2",
            2,
        ),
    );
    wait_for_validator_revocation(&mut fixture).await;

    assert_eq!(
        fixture.worker.validator_for_test(&fixture.post, SOURCE),
        None
    );
    assert!(fixture
        .store
        .present_ranges(fixture.post.as_str())
        .await
        .expect("valid test fixture")
        .is_empty());
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
}

async fn wait_for_validator_revocation(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(EVENT_TIMEOUT, async {
        while fixture
            .worker
            .validator_for_test(&fixture.post, SOURCE)
            .is_some()
        {
            assert!(fixture.step().await);
        }
    })
    .await
    .expect("rejected generation revokes trusted response evidence");
}

fn response(observation: ResponseObservation, validator: &str, order: u64) -> OpenedResponse {
    OpenedResponse::new(
        observation,
        None,
        ResponseWriteMode::Sparse,
        HttpResponseEvidence {
            final_url: SOURCE.into(),
            status: 200,
            content_type: None,
            validator: Some(etag(validator)),
            observed: EvidenceTime::ordered(10, order),
        },
    )
}
