mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::full_disk::{discard, limits, spaced_store};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::pressure_origin::serve;
use delivery_fixture::start_harness_with_store;
use std::sync::Arc;

#[tokio::test]
async fn current_playback_reclaims_cached_media_outside_the_feed_working_set() {
    let mut origin = serve().await;
    let fixture = spaced_store("ghostr-orphan-cache-room", limits(32, 0), 10_000);
    let orphan = sized_item("orphan", &origin.url, 32, 1_000);
    seed_range(&fixture.store, &orphan, 0, &[7; 32]).await;
    let root = fixture.root.clone();
    let harness = start_harness_with_store(
        Arc::new(fixture.store),
        root.clone(),
        DeliveryOptions::default(),
    );
    let current = sized_item("current", &origin.url, 16, 1_000);
    harness.handle.update_focus(focus_now(vec![current], 0, 0));

    tokio::time::timeout(Duration::from_secs(2), origin.wait_for_body())
        .await
        .expect("cold storage must not deadlock current admission");
    origin.release();
    tokio::time::timeout(Duration::from_secs(2), async {
        while harness
            .store
            .present_ranges("current")
            .await
            .expect("valid test fixture")
            .as_slice()
            != core::slice::from_ref(&(0..16))
        {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("current body completes after reclaim");
    assert!(harness
        .store
        .present_ranges("orphan")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert!(harness.store.used_bytes().await <= 32);
    harness.handle.clear().await.expect("valid test fixture");
    drop(harness);
    discard(&root);
}
