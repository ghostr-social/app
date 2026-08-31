use crate::cache::ViewerScope;
use crate::tests::event_cache_support::{note, notes, timestamps, PersistentCacheFixture};
use crate::tests::support::{author, AUTHOR_A, AUTHOR_B};

async fn seed(storage: &PersistentCacheFixture) {
    let cache = storage.cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), vec![note(100)]).await;
}

#[tokio::test]
async fn first_viewer_merges_boot_rows_with_its_durable_rows() {
    let storage = PersistentCacheFixture::new("event-cache-boot-merge");
    seed(&storage).await;
    let cache = storage.cache();
    cache.union(&notes(), vec![note(200)]).await;

    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;

    assert_eq!(
        timestamps(&cache.union(&notes(), Vec::new()).await),
        vec![200, 100]
    );
}

#[tokio::test]
async fn first_viewer_keeps_boot_rows_but_rejects_another_viewers_disk_rows() {
    let storage = PersistentCacheFixture::new("event-cache-boot-isolation");
    seed(&storage).await;
    let cache = storage.cache();
    cache.union(&notes(), vec![note(200)]).await;

    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_B))).await;

    assert_eq!(
        timestamps(&cache.union(&notes(), Vec::new()).await),
        vec![200]
    );
}
