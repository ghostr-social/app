use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseRejection,
    ResponseWriteMode,
};
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};

const SOURCE: &str = "https://unused.example/video.mp4";

#[tokio::test]
async fn malformed_new_generation_revokes_cached_bytes_and_validator() {
    let (parser, mut started) = GatedTimelineParser::new(None, 1);
    let mut fixture = TimelineManagerFixture::new(parser.clone()).await;
    fixture.focus();
    assert!(fixture.worker.step().await);
    assert_eq!(started.recv().await, Some(0));
    parser.release(0);
    let attempt = fixture
        .worker
        .register_response_attempt_for_test(&fixture.post, SOURCE);
    fixture.worker.queue_response_for_test(
        attempt.clone(),
        response(ResponseObservation::Ignored {
            total: None,
            range_support: Some(false),
        }, "v1", 1),
    );
    wait_for_validator(&mut fixture, "v1").await;
    fixture.store.write_range(fixture.post.as_str(), 0, &[7; 4]).await.unwrap();

    fixture.worker.queue_response_for_test(
        attempt,
        response(ResponseObservation::Rejected(ResponseRejection::Semantics), "v2", 2),
    );
    assert!(fixture.worker.step().await);

    assert_eq!(fixture.worker.validator_for_test(&fixture.post, SOURCE), None);
    assert!(fixture.store.present_ranges(fixture.post.as_str()).await.unwrap().is_empty());
    tokio::fs::remove_dir_all(fixture.root).await.unwrap();
}

async fn wait_for_validator(fixture: &mut TimelineManagerFixture, value: &str) {
    let expected = etag(value);
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while fixture.worker.validator_for_test(&fixture.post, SOURCE) != Some(expected.clone()) {
            assert!(fixture.worker.step().await);
        }
    }).await.expect("accepted response evidence arrives");
}

fn response(observation: ResponseObservation, validator: &str, order: u64) -> OpenedResponse {
    OpenedResponse::new(observation, None, ResponseWriteMode::Sparse, HttpResponseEvidence {
        final_url: SOURCE.into(),
        status: 200,
        content_type: None,
        validator: Some(etag(validator)),
        observed: EvidenceTime::ordered(10, order),
    })
}

fn etag(value: &str) -> EvidenceValidator {
    EvidenceValidator::strong_etag(&format!("\"{value}\"")).unwrap()
}
