mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{candidate, focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::adaptive::{AllocationReason, PreemptionAuthority};

#[tokio::test]
async fn relay_projection_has_only_bounded_metadata_authority_before_focus() {
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), std::sync::Arc::clone(&log)).await;
    let harness = start_harness("ghostr-projection-authority", DeliveryOptions::default());
    harness
        .handle
        .admit_candidate(candidate("relay-first", &origin, Some(16), 1));

    tokio::time::sleep(Duration::from_millis(100)).await;
    let projected: Vec<_> = harness
        .handle
        .plan_history()
        .iter()
        .flat_map(|entry| entry.plan.allocations.iter())
        .filter(|allocation| allocation.post.as_str() == "relay-first")
        .cloned()
        .collect();
    assert_eq!(projected.len(), 1, "{projected:#?}");
    assert_eq!(projected[0].reason, AllocationReason::MediaBootstrap);
    assert_eq!(projected[0].authority, PreemptionAuthority::Speculative);

    harness.handle.update_focus(focus_now(
        vec![sized_item("canonical", &origin, 16, 4_000)],
        0,
        0,
    ));
    wait_for_ranges(&harness.store, "canonical", &[(0, 16)]).await;
    assert!(hits(&log).iter().any(|hit| hit.starts_with("origin:GET:")));
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
