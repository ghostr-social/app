use crate::cache::ViewerScope;
use crate::session_generation::SessionGeneration;
use crate::tests::event_cache_support::{note, notes, timestamps, PersistentCacheFixture};
use crate::tests::support::{author, AUTHOR_A};

#[tokio::test]
async fn same_viewer_recovers_verified_events_after_a_cold_restart() {
    let storage = PersistentCacheFixture::new("event-cache-restart");
    let writer = storage.cache();
    writer.adopt(ViewerScope::SignedIn(author(AUTHOR_A))).await;
    writer.union(&notes(), vec![note(100)]).await;
    drop(writer);

    let restored = storage.cache();
    let generation = SessionGeneration::initial().next();
    restored
        .reset_session_for(generation, ViewerScope::SignedIn(author(AUTHOR_A)))
        .await;

    let events = restored
        .stored_for(generation, &notes())
        .await
        .expect("current generation");
    assert_eq!(timestamps(&events), vec![100]);
}
