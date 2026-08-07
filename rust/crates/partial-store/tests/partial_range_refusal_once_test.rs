//! When there is genuinely nothing left to give back, the store decides
//! once. A player pulling every buffer must not re-run eviction and
//! re-report the same answer sixteen times a second; a fresh
//! measurement is what starts a new decision.

mod store_fixture;

use std::time::Duration;
use store_fixture::{discard, limits, paced_store};

const RECHECK: Duration = Duration::from_secs(2);

#[tokio::test(start_paused = true)]
async fn partial_range_refusal_is_decided_once_per_measurement() {
    let fixture = paced_store(
        "ghostr-refuse-once",
        limits(u64::MAX, 1_000),
        1_000,
        RECHECK,
    );
    let store = &fixture.store;

    for buffer in 0..16_u64 {
        let refused = store
            .write_range("hot", buffer * 100, &[7; 100])
            .await
            .expect_err("nothing above the reserve is spendable");
        assert!(
            refused.to_string().contains("space"),
            "unhelpful: {refused}"
        );
    }

    assert_eq!(store.refusals(), 1, "one decision, not sixteen");
    assert_eq!(*fixture.used_bytes.lock().await, 0);
    assert!(!fixture.root.exists(), "a refused write creates no file");

    fixture.space.set(1_400);
    tokio::time::advance(RECHECK * 2).await;
    store
        .write_range("hot", 0, &[7; 100])
        .await
        .expect("a new measurement admits the write");
    assert_eq!(store.refusals(), 1, "a granted write decides nothing");

    discard(&fixture.root);
}
