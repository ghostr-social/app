use crate::cache::ViewerScope;
use crate::tests::event_cache_support::{note, notes, timestamps, PersistentCacheFixture};
use crate::tests::support::{author, AUTHOR_A, AUTHOR_B};

async fn seed(storage: &PersistentCacheFixture) {
    let cache = storage.cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), vec![note(100)]).await;
}

#[tokio::test]
async fn another_viewer_cannot_restore_the_previous_viewers_events() {
    let storage = PersistentCacheFixture::new("event-cache-other-viewer");
    seed(&storage).await;

    let cache = storage.cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_B))).await;

    assert!(cache.union(&notes(), Vec::new()).await.is_empty());
}

#[tokio::test]
async fn signed_out_cannot_restore_signed_in_events() {
    let storage = PersistentCacheFixture::new("event-cache-signed-out");
    seed(&storage).await;

    let cache = storage.cache();
    cache.adopt(ViewerScope::SignedOut).await;

    assert!(cache.union(&notes(), Vec::new()).await.is_empty());
}

#[tokio::test]
async fn signed_out_restores_only_its_own_events() {
    let storage = PersistentCacheFixture::new("event-cache-signed-out-restart");
    let writer = storage.cache();
    writer.adopt(ViewerScope::SignedOut).await;
    writer.union(&notes(), vec![note(200)]).await;
    drop(writer);

    let restored = storage.cache();
    restored.adopt(ViewerScope::SignedOut).await;

    assert_eq!(
        timestamps(&restored.union(&notes(), Vec::new()).await),
        vec![200]
    );
}
