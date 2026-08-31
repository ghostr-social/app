use crate::cache::ViewerScope;
use crate::tests::event_cache_support::{note, notes, persistent_cache, PersistentCacheFixture};
use crate::tests::failing_wipe_database::FailingWipeDatabase;
use crate::tests::support::{author, AUTHOR_A, AUTHOR_B};
use std::sync::Arc;

#[tokio::test]
async fn failed_wipe_cannot_admit_or_persist_the_previous_viewers_rows() {
    let storage = PersistentCacheFixture::new("event-cache-failed-wipe");
    let cache = persistent_cache(storage.root(), Arc::new(FailingWipeDatabase::new()));
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), vec![note(100)]).await;

    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_B))).await;

    assert!(cache.union(&notes(), Vec::new()).await.is_empty());
    drop(cache);
    let restarted = storage.cache();
    restarted
        .adopt(ViewerScope::SignedIn(author(AUTHOR_B)))
        .await;
    assert!(restarted.union(&notes(), Vec::new()).await.is_empty());
}
