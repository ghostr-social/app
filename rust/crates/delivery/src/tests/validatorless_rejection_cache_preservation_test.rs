use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseRejection, ResponseWriteMode,
};
use crate::tests::response_evidence_fixture::{etag, wait_for_validator, SOURCE};
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};

#[tokio::test]
async fn malformed_validatorless_response_preserves_trusted_cached_bytes() {
    let (parser, mut started) = GatedTimelineParser::new(None, 1);
    let mut fixture =
        TimelineManagerFixture::new(std::sync::Arc::<GatedTimelineParser>::clone(&parser)).await;
    fixture.focus();
    assert!(fixture.step().await);
    assert_eq!(started.recv().await, Some(0));
    parser.release(0);
    let attempt = fixture
        .worker
        .register_response_attempt_for_test(&fixture.post, SOURCE);
    fixture
        .worker
        .queue_response_for_test(attempt.clone(), accepted());
    wait_for_validator(&mut fixture, etag("v1")).await;
    fixture
        .store
        .write_range(fixture.post.as_str(), 0, &[7; 4])
        .await
        .expect("valid test fixture");

    fixture.worker.queue_response_for_test(attempt, rejected());
    assert!(fixture.step().await);

    assert_eq!(
        fixture.worker.validator_for_test(&fixture.post, SOURCE),
        Some(etag("v1"))
    );
    let ranges = fixture
        .store
        .present_ranges(fixture.post.as_str())
        .await
        .expect("valid test fixture");
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], 0..4);
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
}

fn accepted() -> OpenedResponse {
    response(
        ResponseObservation::Ignored {
            total: None,
            range_support: Some(false),
        },
        SOURCE,
        Some(etag("v1")),
        1,
    )
}

fn rejected() -> OpenedResponse {
    response(
        ResponseObservation::Rejected(ResponseRejection::Semantics),
        "https://redirect.example/video.mp4",
        None,
        2,
    )
}

fn response(
    observation: ResponseObservation,
    final_url: &str,
    validator: Option<EvidenceValidator>,
    order: u64,
) -> OpenedResponse {
    OpenedResponse::new(
        observation,
        None,
        ResponseWriteMode::Sparse,
        HttpResponseEvidence {
            final_url: final_url.into(),
            status: 200,
            content_type: None,
            validator,
            observed: EvidenceTime::ordered(10, order),
        },
    )
}
