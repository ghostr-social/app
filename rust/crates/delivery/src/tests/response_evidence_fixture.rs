use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use core::time::Duration;
use ghostr_engine::evidence::EvidenceValidator;

pub(super) const SOURCE: &str = "https://unused.example/video.mp4";
pub(super) const EVENT_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn wait_for_validator(
    fixture: &mut TimelineManagerFixture,
    expected: EvidenceValidator,
) {
    tokio::time::timeout(EVENT_TIMEOUT, async {
        while fixture.worker.validator_for_test(&fixture.post, SOURCE) != Some(expected.clone()) {
            assert!(fixture.step().await);
        }
    })
    .await
    .expect("response evidence is absorbed");
}

pub(super) fn etag(value: &str) -> EvidenceValidator {
    EvidenceValidator::strong_etag(format!("\"{value}\""))
        .expect("valid response-evidence fixture")
}
