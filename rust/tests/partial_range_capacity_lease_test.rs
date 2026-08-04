mod store_space;

use store_space::{discard, limits, spaced_store};

/// Eviction must never pull the file out from under a reader, so a
/// leased video is skipped even when it is the oldest candidate.
#[tokio::test]
async fn partial_range_capacity_eviction_spares_a_leased_video() {
    let fixture = spaced_store("ghostr-cap-lease", limits(1_000_000, 1_000), 3_000);
    let store = &fixture.store;
    store.write_range("held", 0, &[1; 400]).await.expect("held");
    store.write_range("free", 0, &[2; 400]).await.expect("free");
    let lease = store.lease("held");

    fixture.space.set(600);
    let evicted = store.enforce_capacity().await;

    assert_eq!(evicted, 400);
    assert_eq!(
        store.present_ranges("held").await.expect("held ranges"),
        vec![0..400],
        "the leased video stays despite being the oldest"
    );
    assert_eq!(
        store.present_ranges("free").await.expect("free ranges"),
        Vec::new(),
        "the unleased newer video is evicted instead"
    );

    drop(lease);
    assert_eq!(
        store.enforce_capacity().await,
        400,
        "the lease was the only thing holding it"
    );
    assert_eq!(*fixture.used_bytes.lock().await, 0);
    assert!(!fixture.root.join("held.part").exists());

    discard(&fixture.root);
}
