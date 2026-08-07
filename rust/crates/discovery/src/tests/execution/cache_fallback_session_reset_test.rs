//! Offline fallback never leaks cached rows across account sessions.

use crate::execution::cache_fallback::cached_or_failure;
use crate::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use crate::tests::event_cache_support::{cache, notes};

#[tokio::test]
async fn stale_session_fallback_reports_the_reset() {
    let cache = cache();
    let stale = SessionGeneration::initial();
    cache.reset_session(stale.next()).await;

    let failure = cached_or_failure(&cache, stale, &notes(), "offline")
        .await
        .expect_err("a stale query cannot read the new session");

    assert_eq!(failure.message, SESSION_RESET_MESSAGE);
}
