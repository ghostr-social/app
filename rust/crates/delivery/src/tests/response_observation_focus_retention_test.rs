use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use crate::tests::response_observation_focus_fixture::replacement_focus;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};

const SOURCE: &str = "https://unused.example/video.mp4";

#[tokio::test]
async fn queued_headers_survive_focus_pruning_until_the_attempt_is_terminal() {
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
        .queue_response_for_test(attempt.clone(), response());
    fixture.handle.update_focus(replacement_focus());

    assert!(fixture.worker.step().await);
    assert!(fixture.worker.catalog_contains_for_test(&fixture.post));
    wait_for_validator(&mut fixture).await;
    assert_eq!(
        fixture.worker.validator_for_test(&fixture.post, SOURCE),
        Some(etag())
    );

    fixture
        .worker
        .queue_cancelled_attempt_for_test(attempt, SOURCE);
    wait_for_prune(&mut fixture).await;
    assert!(!fixture.worker.catalog_contains_for_test(&fixture.post));
    tokio::fs::remove_dir_all(fixture.root).await.unwrap();
}

async fn wait_for_validator(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while fixture.worker.validator_for_test(&fixture.post, SOURCE) != Some(etag()) {
            assert!(fixture.worker.catalog_contains_for_test(&fixture.post));
            assert!(fixture.worker.step().await);
        }
    })
    .await
    .expect("queued response evidence is absorbed");
}

async fn wait_for_prune(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while fixture.worker.catalog_contains_for_test(&fixture.post) {
            assert!(fixture.worker.step().await);
        }
    })
    .await
    .expect("terminal acknowledgement releases evidence retention");
}

fn response() -> OpenedResponse {
    OpenedResponse::new(
        ResponseObservation::Ignored {
            total: None,
            range_support: Some(false),
        },
        None,
        ResponseWriteMode::Sparse,
        HttpResponseEvidence {
            final_url: SOURCE.into(),
            status: 200,
            content_type: Some("video/mp4".into()),
            validator: Some(etag()),
            observed: EvidenceTime::ordered(10, 1),
        },
    )
}

fn etag() -> EvidenceValidator {
    EvidenceValidator::strong_etag("\"v1\"").unwrap()
}
