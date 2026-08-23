use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseRejection,
    ResponseWriteMode,
};
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};

const SOURCE: &str = "https://unused.example/video.mp4";

#[tokio::test]
async fn rejected_error_page_cannot_replace_media_validator() {
    let (parser, mut started) = GatedTimelineParser::new(None, 1);
    let mut fixture = TimelineManagerFixture::new(parser.clone()).await;
    fixture.focus();
    assert!(fixture.worker.step().await);
    assert_eq!(started.recv().await, Some(0));
    parser.release(0);
    let attempt = fixture
        .worker
        .register_response_attempt_for_test(&fixture.post, SOURCE);
    fixture
        .worker
        .queue_response_for_test(attempt.clone(), accepted());
    wait_for_validator(&mut fixture).await;
    assert_eq!(fixture.worker.validator_for_test(&fixture.post, SOURCE), Some(etag()));

    fixture.worker.queue_response_for_test(attempt, rejected());
    assert!(fixture.worker.step().await);

    assert_eq!(fixture.worker.validator_for_test(&fixture.post, SOURCE), Some(etag()));
    tokio::fs::remove_dir_all(fixture.root).await.unwrap();
}

async fn wait_for_validator(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while fixture.worker.validator_for_test(&fixture.post, SOURCE) != Some(etag()) {
            assert!(fixture.worker.step().await);
        }
    })
    .await
    .expect("accepted response evidence arrives");
}

fn accepted() -> OpenedResponse {
    response(
        ResponseObservation::Ignored {
            total: None,
            range_support: Some(false),
        },
        200,
        Some(etag()),
        1,
    )
}

fn rejected() -> OpenedResponse {
    response(
        ResponseObservation::Rejected(ResponseRejection::Status),
        503,
        None,
        2,
    )
}

fn response(
    observation: ResponseObservation,
    status: u16,
    validator: Option<EvidenceValidator>,
    order: u64,
) -> OpenedResponse {
    OpenedResponse::new(
        observation,
        None,
        ResponseWriteMode::Sparse,
        HttpResponseEvidence {
            final_url: SOURCE.into(),
            status,
            content_type: None,
            validator,
            observed: EvidenceTime::ordered(10, order),
        },
    )
}

fn etag() -> EvidenceValidator {
    EvidenceValidator::strong_etag("\"v1\"").unwrap()
}
