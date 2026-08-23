use crate::probe::media::ProbeResult;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};
use std::time::Duration;

const SOURCE: &str = "https://unused.example/video.mp4";
const REDIRECTED: &str = "https://cdn.example/video.mp4";

#[tokio::test]
async fn validated_lengthless_head_installs_durable_http_generation() {
    let (parser, mut started) = GatedTimelineParser::new(None, 1);
    let mut fixture = TimelineManagerFixture::new(parser.clone()).await;
    fixture.focus();
    assert!(fixture.worker.step().await);
    assert_eq!(started.recv().await, Some(0));
    parser.release(0);

    fixture.worker.finish_probe_result_for_test(
        &fixture.post,
        SOURCE,
        result(),
    ).await.expect("current probe identity");

    let json = tokio::fs::read_to_string(
        fixture.root.join("post.http-generation.json"),
    ).await.unwrap();
    let stored: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(stored["key"]["final_url"], REDIRECTED);
    assert!(fixture.store.present_ranges(fixture.post.as_str()).await.unwrap().is_empty());
    tokio::fs::remove_dir_all(fixture.root).await.unwrap();
}

fn result() -> ProbeResult {
    ProbeResult {
        final_url: REDIRECTED.into(),
        observed: EvidenceTime::ordered(10, 1),
        content_length: None,
        accept_ranges: Some(true),
        content_type: Some("video/mp4".into()),
        validator: EvidenceValidator::strong_etag("\"v1\""),
        ttfb: Duration::from_millis(10),
    }
}
