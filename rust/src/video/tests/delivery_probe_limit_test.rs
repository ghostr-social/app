use crate::engine::catalog::Catalog;
use crate::engine::{DeliveryKind, PostId, VideoMeta};
use crate::video::delivery_probes::ProbeBook;
use crate::video::delivery_retry::{RetryBook, RetryPolicy};

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
    let mut probes = ProbeBook::new(1);

    let claimed = probes.claim(&catalog, &posts, &retry);

    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].0, posts[0]);
}
