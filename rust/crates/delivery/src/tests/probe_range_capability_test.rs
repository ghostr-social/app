use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::catalog::{Catalog, LearnedFacts};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn known_size_media_is_still_probed_until_its_range_capability_is_known() {
    let post = PostId::new("known-size");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), metadata());
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);

    let claimed = probes.claim(&catalog, std::slice::from_ref(&post), &retry);
    assert_eq!(claimed.len(), 1);
    probes.release(&post);
    catalog.learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    assert!(probes.claim(&catalog, &[post], &retry).is_empty());
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(1_000_000),
        duration_ms: Some(8_000),
    }
}
