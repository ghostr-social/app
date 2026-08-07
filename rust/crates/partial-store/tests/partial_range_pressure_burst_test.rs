//! Device pass 3 recorded sixteen "video store is out of space"
//! refusals inside one second, right after the store had given back far
//! more than it needed. Bytes handed back to the file system are free
//! space again: a measurement taken before an eviction must not go on
//! refusing writes the store has just made room for.

mod store_space;

use std::time::Duration;
use store_space::{discard, limits, paced_store};

const RECHECK: Duration = Duration::from_secs(2);

#[tokio::test(start_paused = true)]
async fn partial_range_writes_resume_after_the_store_gives_bytes_back() {
    let fixture = paced_store(
        "ghostr-pressure-burst",
        limits(u64::MAX, 1_000),
        5_000,
        RECHECK,
    );
    let store = &fixture.store;
    store
        .write_range("cold", 0, &[1; 2_000])
        .await
        .expect("cold");
    store
        .write_range("hot", 0, &[2; 100])
        .await
        .expect("hot head");

    fixture.space.set(900); // another app filled the device
    tokio::time::advance(RECHECK * 2).await;
    assert_eq!(
        store.enforce_capacity().await,
        2_000,
        "the coldest video is given back to protect the reserve"
    );

    for buffer in 0..8_u64 {
        store
            .write_range("hot", 100 + buffer * 100, &[2; 100])
            .await
            .unwrap_or_else(|error| panic!("refused buffer {buffer} after making room: {error}"));
    }

    assert_eq!(store.refusals(), 0, "no refusal was ever decided");
    assert_eq!(*fixture.used_bytes.lock().await, 900);
    assert_eq!(
        store.present_ranges("hot").await.expect("ranges"),
        vec![0..900]
    );
    discard(&fixture.root);
}
