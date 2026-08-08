use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn probe_claims_respect_the_configured_concurrency_limit() {
    let posts = [PostId::new("first"), PostId::new("second")];
    let mut catalog = Catalog::new();
    for post in &posts {
        catalog.upsert(
            post.clone(),
            VideoMeta {
                urls: vec![format!("https://media.example/{}.mp4", post.as_str())],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: None,
                duration_ms: None,
            },
        );
    }
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);

    let claimed = probes.claim(&catalog, &posts, &retry);

    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].0, posts[0]);
}
