use crate::segmented::prepare::PreparedObject;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

#[test]
fn mirror_retry_discards_objects_and_bytes_from_the_failed_source() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["primary".to_owned()])]);
    assert!(cache.mark_stage_preparing(&post, 1, 500, manifest_maximum()));
    assert_eq!(
        cache.store_stage_object(&post, 1, object("primary", b"old")),
        Some(3)
    );

    assert!(cache.reset_stage_retry(&post, 1));

    assert_eq!(cache.snapshot("post").phase, SegmentedPhase::Queued);
    assert_eq!(cache.snapshot("post").bytes_present, 0);
    assert!(cache.object("primary").is_none());

    assert!(cache.mark_stage_preparing(&post, 1, 500, manifest_maximum()));
    assert_eq!(
        cache.store_stage_object(&post, 1, object("mirror", b"new")),
        Some(3)
    );
    assert!(cache.mark_stage_ready(&post, 1));
    assert!(cache.object("primary").is_none());
    assert_eq!(cache.object("mirror").unwrap().body.as_ref(), b"new");
}

#[test]
fn staged_root_is_invisible_until_the_entire_bootstrap_is_ready() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["root".to_owned()])]);
    assert!(cache.mark_stage_preparing(&post, 1, 500, manifest_maximum()));
    assert_eq!(
        cache.store_stage_object(&post, 1, object("root", b"manifest")),
        Some(8)
    );

    assert!(cache.object("root").is_none());
    assert!(cache.mark_stage_ready(&post, 1));
    assert_eq!(cache.object("root").unwrap().body.as_ref(), b"manifest");
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

fn manifest_maximum() -> u64 {
    ghostr_engine::adaptive::HlsBootstrapStage::RootManifest.maximum_bytes()
}
