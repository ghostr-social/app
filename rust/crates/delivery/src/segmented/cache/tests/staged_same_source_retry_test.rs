use crate::segmented::prepare::PreparedObject;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

#[test]
fn same_source_retry_releases_reservation_but_preserves_prior_stages() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["root".to_owned()])]);
    assert!(cache.mark_stage_preparing(
        &post,
        1,
        500,
        HlsBootstrapStage::RootManifest.maximum_bytes()
    ));
    cache
        .store_stage_object(&post, 1, object("root", b"manifest"))
        .unwrap();
    assert!(cache.mark_stage_preparing(
        &post,
        1,
        500,
        HlsBootstrapStage::Initialization.maximum_bytes()
    ));

    assert!(cache.release_stage_attempt(&post, 1));

    let snapshot = cache.snapshot("post");
    assert_eq!(snapshot.phase, SegmentedPhase::Queued);
    assert_eq!(snapshot.bytes_present, 8);
    assert!(cache.mark_stage_preparing(
        &post,
        1,
        500,
        HlsBootstrapStage::Initialization.maximum_bytes()
    ));
}

fn object(request_url: &str, body: &[u8]) -> PreparedObject {
    PreparedObject {
        request_url: request_url.to_owned(),
        final_url: Url::parse("https://primary.example/index.m3u8").unwrap(),
        body: Arc::from(body),
        content_type: Some("application/vnd.apple.mpegurl".to_owned()),
        cache: Default::default(),
    }
}
