use crate::probe::media::ProbeResult;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use core::time::Duration;
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, VideoMeta};
use std::sync::Arc;

const SOURCE: &str = "https://unused.example/video.mp4";
const MIRROR: &str = "https://mirror.example/video.mp4";

#[tokio::test]
async fn mirror_head_cannot_replace_retained_canonical_bytes() {
    let (parser, _) = GatedTimelineParser::new(None, 1);
    let mut fixture = TimelineManagerFixture::new(Arc::<GatedTimelineParser>::clone(&parser)).await;
    fixture.focus_with(VideoMeta {
        urls: vec![SOURCE.into(), MIRROR.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(2_048),
        duration_ms: Some(1_000),
    });
    assert!(fixture.step().await, "focus accepted");
    let binding = fixture
        .store
        .representation_binding("post")
        .await
        .expect("binding");
    let identity = binding.transfer(SOURCE).expect("canonical identity");
    fixture
        .store
        .select_transfer(identity.clone())
        .await
        .expect("selected source");
    let generation = SourceGeneration::try_new(SOURCE, "\"canonical\"", 2_048).expect("generation");
    fixture
        .store
        .accept_generation(&identity, generation.clone())
        .await
        .expect("canonical generation");
    assert!(
        fixture
            .store
            .write_range_for_generation_if_current(&identity, &generation, 0, b"0123")
            .await
            .expect("canonical prefix"),
        "current write accepted"
    );

    fixture
        .worker
        .finish_probe_result_for_test(&fixture.post, MIRROR, result())
        .await
        .expect("mirror belongs to current representation");

    assert_eq!(
        fixture
            .store
            .read_range("post", 0..4)
            .await
            .expect("read prefix"),
        Some(b"0123".to_vec())
    );
    std::fs::remove_dir_all(fixture.root).ok();
}

fn result() -> ProbeResult {
    ProbeResult {
        request_selection: None,
        final_url: MIRROR.into(),
        observed: EvidenceTime::ordered(10, 1),
        content_length: Some(2_048),
        accept_ranges: Some(true),
        content_type: Some("video/mp4".into()),
        validator: EvidenceValidator::strong_etag("\"mirror\""),
        ttfb: Duration::from_millis(10),
    }
}
