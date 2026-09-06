//! Existing probe bytes must not delay a last-modified response until EOF.
mod continuous_body_fixture;
mod delivery_fixture;
#[path = "continuous_body_fixture/probe.rs"]
mod probe;

use continuous_body_fixture::{serve, wait_at, TOTAL};
use delivery_fixture::items::{focus_now, sized_item};
use probe::seed_probe;

#[tokio::test]
async fn dated_whole_response_streams_beyond_an_existing_probe_before_eof() {
    let origin = serve(true).await;
    let harness = continuous_body_fixture::harness("continuous-dated-probe");
    let item = sized_item("current", &origin.url, TOTAL, 60_000);
    seed_probe(&harness.store, &item).await;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    let progress = tokio::time::timeout(
        core::time::Duration::from_secs(20),
        wait_at(&harness.store, 4 * 1024 * 1024),
    )
    .await;
    harness.handle.update_focus(focus_now(Vec::new(), 0, 0));
    assert!(
        progress.is_ok(),
        "matching whole bytes must stream before the withheld EOF"
    );
    assert!(!harness
        .store
        .is_complete("current")
        .await
        .expect("incomplete response"));
    assert_eq!(
        origin
            .whole_requests
            .load(core::sync::atomic::Ordering::Relaxed),
        1
    );
}
