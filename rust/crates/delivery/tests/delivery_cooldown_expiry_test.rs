mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::transient_origin::{body_count, serve, Attempts};
use tokio::time::Instant;

#[tokio::test]
async fn failed_focused_download_resumes_when_its_cooldown_expires() {
    let (url, attempts) = serve().await;
    let harness = start_harness("ghostr-cooldown-expiry", DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![sized_item("focused", &url, 16, 1_000)],
        0,
        0,
    ));

    wait_for_attempts(&attempts, 3).await;

    assert_eq!(body_count(&attempts), 3);
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_attempts(attempts: &Attempts, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while body_count(attempts) < expected {
        assert!(Instant::now() < deadline, "cooldown did not resume retry");
        tokio::task::yield_now().await;
    }
}
