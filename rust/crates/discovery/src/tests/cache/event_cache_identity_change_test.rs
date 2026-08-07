//! Identity change must not leak one viewer's session pool into the
//! next viewer's feed. The engine outlives a sign-out — the gateway and
//! its client are installed once per process — so the pool is scoped to
//! whoever the main feed named last, and a change empties it.

use crate::cache::ViewerScope;
use crate::tests::event_cache_support::{cache, note, notes};
use crate::tests::support::{author, AUTHOR_A, AUTHOR_B};

#[tokio::test]
async fn signing_in_as_someone_else_drops_the_previous_pool() {
    let cache = cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), vec![note(100)]).await;

    let dropped = cache.adopt(ViewerScope::SignedIn(author(AUTHOR_B))).await;

    assert!(dropped, "a new viewer empties the pool");
    assert!(cache.union(&notes(), Vec::new()).await.is_empty());
}

#[tokio::test]
async fn signing_out_drops_the_pool_too() {
    let cache = cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), vec![note(100)]).await;

    let dropped = cache.adopt(ViewerScope::SignedOut).await;

    assert!(dropped);
    assert!(cache.union(&notes(), Vec::new()).await.is_empty());
}

#[tokio::test]
async fn reopening_a_feed_for_the_same_viewer_keeps_the_pool() {
    let cache = cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), vec![note(100)]).await;

    let dropped = cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;

    assert!(!dropped);
    assert_eq!(cache.union(&notes(), Vec::new()).await.len(), 1);
}
