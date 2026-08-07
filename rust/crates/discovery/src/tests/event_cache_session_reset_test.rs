//! A late fetch from the old account cannot refill the reset event pool.

use crate::session_generation::SessionGeneration;
use crate::tests::event_cache_support::{cache, note, notes, timestamps};

#[tokio::test]
async fn stale_session_writes_are_rejected_after_reset() {
    let cache = cache();
    let stale = SessionGeneration::initial();
    cache.remember_for(stale, &[note(100)]).await;
    let fresh = stale.next();

    cache.reset_session(fresh).await;
    let accepted = cache.remember_for(stale, &[note(200)]).await;

    assert!(!accepted, "the late old-account write must be rejected");
    assert!(
        cache
            .union_for(stale, &notes(), vec![note(200)])
            .await
            .is_none(),
        "the late old-account read must be rejected",
    );
    assert!(cache.stored(&notes()).await.is_empty());
    assert!(cache.remember_for(fresh, &[note(300)]).await);
    assert_eq!(timestamps(&cache.stored(&notes()).await), vec![300]);
}
