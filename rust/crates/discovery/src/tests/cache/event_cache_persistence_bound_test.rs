use crate::cache::persistence::MAX_SNAPSHOT_BYTES;
use crate::cache::ViewerScope;
use crate::tests::event_cache_support::{note_with_content, notes, PersistentCacheFixture};
use crate::tests::support::{author, AUTHOR_A};

#[tokio::test]
async fn durable_snapshot_never_exceeds_its_byte_budget() {
    let storage = PersistentCacheFixture::new("event-cache-byte-bound");
    let cache = storage.cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    let oversized = note_with_content(100, "x".repeat(MAX_SNAPSHOT_BYTES + 1));

    assert_eq!(
        cache.union(&notes(), vec![oversized.clone()]).await,
        vec![oversized]
    );
    assert!(
        std::fs::metadata(storage.snapshot())
            .expect("snapshot")
            .len()
            <= MAX_SNAPSHOT_BYTES as u64
    );
}
