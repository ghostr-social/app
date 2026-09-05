use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use crate::tests::response_evidence_fixture::{etag, wait_for_validator, EVENT_TIMEOUT, SOURCE};
use crate::tests::response_observation_focus_fixture::replacement_focus;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::evidence::EvidenceTime;

#[tokio::test]
async fn queued_headers_survive_focus_pruning_until_the_attempt_is_terminal() {
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
        .queue_response_for_test(attempt.clone(), response());
    fixture.handle.update_focus(replacement_focus());

    assert!(fixture.step().await);
    assert!(fixture.worker.catalog_contains_for_test(&fixture.post));
    wait_for_validator(&mut fixture, etag("v1")).await;
    assert_eq!(
        fixture.worker.validator_for_test(&fixture.post, SOURCE),
        Some(etag("v1"))
    );

    fixture
        .worker
        .queue_cancelled_attempt_for_test(attempt, SOURCE);
    wait_for_prune(&mut fixture).await;
    assert!(!fixture.worker.catalog_contains_for_test(&fixture.post));
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("fixture");
}

async fn wait_for_prune(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(EVENT_TIMEOUT, async {
        while fixture.worker.catalog_contains_for_test(&fixture.post) {
            assert!(fixture.step().await);
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
            request_selection: None,
            final_url: SOURCE.into(),
            status: 200,
            content_type: Some("video/mp4".into()),
            validator: Some(etag("v1")),
            observed: EvidenceTime::ordered(10, 1),
        },
    )
}
