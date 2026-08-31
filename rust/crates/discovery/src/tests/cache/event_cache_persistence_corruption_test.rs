use crate::cache::ViewerScope;
use crate::tests::event_cache_support::{note, notes, PersistentCacheFixture};
use crate::tests::support::{author, AUTHOR_A};

async fn restore(storage: &PersistentCacheFixture) -> Vec<nostr_sdk::Event> {
    let cache = storage.cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), Vec::new()).await
}

#[tokio::test]
async fn truncated_snapshot_fails_closed() {
    let storage = PersistentCacheFixture::new("event-cache-truncated");
    std::fs::create_dir_all(storage.root()).expect("fixture directory");
    std::fs::write(storage.snapshot(), br#"{"version":1,"viewer""#).expect("fixture snapshot");

    assert!(restore(&storage).await.is_empty());
}

#[tokio::test]
async fn serialized_event_is_verified_before_restore() {
    let storage = PersistentCacheFixture::new("event-cache-unverified");
    let writer = storage.cache();
    writer.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    writer.union(&notes(), vec![note(100)]).await;
    drop(writer);
    let mut snapshot: serde_json::Value =
        serde_json::from_slice(&std::fs::read(storage.snapshot()).expect("snapshot"))
            .expect("snapshot JSON");
    snapshot["events"][0]["content"] = serde_json::json!("tampered");
    std::fs::write(
        storage.snapshot(),
        serde_json::to_vec(&snapshot).expect("JSON"),
    )
    .expect("tampered snapshot");

    assert!(restore(&storage).await.is_empty());
}
