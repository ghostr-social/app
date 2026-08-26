//! Reliable metadata permits an exact lengthless whole-body acquisition above the bootstrap cap.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::probe_origins::serve_recording_range_blind_body;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::adaptive::REQUEST_SLICE_BYTES;

#[tokio::test]
async fn reliable_size_completes_a_lengthless_body_larger_than_the_speculative_cap() {
    let body = vec![b'k'; REQUEST_SLICE_BYTES as usize + 1];
    let log = hit_log();
    let origin = serve_recording_range_blind_body(std::sync::Arc::clone(&log), body.clone()).await;
    let harness = start_harness("ghostr-known-lengthless-whole", DeliveryOptions::default());

    harness.handle.update_focus(focus_now(
        vec![sized_item("aa11", &origin, body.len() as u64, 10_000)],
        0,
        0,
    ));

    wait_for_ranges(&harness.store, "aa11", &[(0, body.len() as u64)]).await;
    assert_eq!(
        harness
            .store
            .read_range("aa11", 0..body.len() as u64)
            .await
            .expect("valid test fixture")
            .expect("valid test fixture"),
        body
    );
    let requests = hits(&log);
    assert_eq!(
        requests
            .iter()
            .filter(|request| *request == "GET:full")
            .count(),
        1,
        "exact whole request repeated: {requests:?}"
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
