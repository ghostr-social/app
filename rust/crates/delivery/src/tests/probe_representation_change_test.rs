use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn representation_change_clears_completed_probe_history() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec!["https://media.example/video.mp4".into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    );
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    let identity = catalog
        .transfer_identity(&post, "https://media.example/video.mp4")
        .unwrap();
    probes.learned(&identity, None);
    assert!(probes
        .claim(&catalog, std::slice::from_ref(&post), &retry)
        .is_empty());

    probes.representation_changed(&post);

    assert_eq!(probes.claim(&catalog, &[post], &retry).len(), 1);
}
