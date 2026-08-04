//! Search, hashtag and profile feeds name no viewer, and the boot
//! subscription runs before any feed opens. Neither may empty the pool:
//! only a request that actually names a *different* session does.

use crate::discovery::event_cache::ViewerScope;
use crate::discovery::tests::event_cache_support::{cache, note, notes};
use crate::discovery::tests::support::{author, AUTHOR_A};

#[tokio::test]
async fn a_request_that_names_no_viewer_never_drops_the_pool() {
    let cache = cache();
    cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    cache.union(&notes(), vec![note(100)]).await;

    let dropped = cache.adopt(ViewerScope::Unknown).await;

    assert!(!dropped);
    assert_eq!(cache.union(&notes(), Vec::new()).await.len(), 1);
}

#[tokio::test]
async fn the_first_viewer_of_a_session_keeps_what_booting_gathered() {
    let cache = cache();
    cache.union(&notes(), vec![note(100)]).await;

    let dropped = cache.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;

    assert!(!dropped, "nobody else's rows can be in a fresh pool");
    assert_eq!(cache.union(&notes(), Vec::new()).await.len(), 1);
}
