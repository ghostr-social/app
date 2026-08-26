use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn delayed_probe_facts_are_rejected_after_source_replacement() {
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta("https://a.example/video"));
    let mut probes = MetadataProbePool::new(1);
    let retry = RetryBook::new(RetryPolicy::default());
    assert_eq!(
        probes
            .claim(&catalog, core::slice::from_ref(&post), &retry)
            .len(),
        1
    );

    catalog.upsert(post.clone(), meta("https://b.example/video"));

    assert!(probes
        .current_identity(&catalog, &post, "https://a.example/video")
        .is_none());
}

fn meta(url: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![url.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
