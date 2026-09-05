mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::playback::{playing, wait_for_admission};
use delivery_fixture::start_harness;

const VIDEO_BYTES: u64 = 65_536;

#[tokio::test]
async fn same_origin_reserve_waits_for_current_response_completion() {
    let mut origin = ControlledOrigin::serve(VIDEO_BYTES).await;
    let harness = start_harness(
        "ghostr-same-authority-ready",
        production_geometry_parallel_options(),
    );
    let items = ["current", "next", "third"]
        .map(|id| sized_item(id, &origin.url_for(id), VIDEO_BYTES, 4_000));
    harness.handle.update_focus(focus_now(items.to_vec(), 0, 0));
    let current = tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("current request starts");
    assert_eq!(current.path, "/current.mp4");
    harness
        .handle
        .report_playback(playing("current", Duration::from_secs(20)));
    wait_for_admission(&harness.handle).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(100), origin.next())
            .await
            .is_err(),
        "an open response consumes the origin's only request slot"
    );
    assert!(
        current.send_bytes(VIDEO_BYTES as usize).await,
        "current body arrives"
    );
    drop(current);
    let next = tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("immediate next starts after current completes");
    assert_eq!(next.path, "/next.mp4");
    assert_eq!(next.range, 0..VIDEO_BYTES);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), origin.next())
            .await
            .is_err(),
        "future work cannot bypass per-origin admission"
    );
    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).expect("remove fixture");
}
