mod delivery_fixture;
#[path = "delivery_network_class_evidence_test/wait.rs"]
mod wait;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::wait::wait_for_ranges;
use delivery_fixture::{start_harness, DeliveryHarness};
use ghostr_delivery::delivery_events::{DeliveryNetworkStatus, FocusAdmission};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};

#[tokio::test]
async fn manager_persists_wifi_and_cellular_origin_evidence_separately() {
    tokio::time::timeout(core::time::Duration::from_secs(20), exercise_manager())
        .await
        .expect("network-class evidence deadline");
}

async fn exercise_manager() {
    let origin = serve_recording("network", media_body(), hit_log()).await;
    let harness = start_harness("network-class-evidence", DeliveryOptions::default());
    transfer(&harness, "wifi", &origin, NetworkClass::Wifi, 1).await;
    transfer(&harness, "cellular", &origin, NetworkClass::Cellular, 2).await;
    let stats_path = harness.root.join("host_stats.json");
    let root = harness.root.clone();
    drop(harness);
    let stats = wait::for_network_evidence(&stats_path, &origin).await;
    assert_observed(&stats, &origin, NetworkClass::Wifi);
    assert_observed(&stats, &origin, NetworkClass::Cellular);
    assert_unavailable_is_cold(&stats, &origin);
    std::fs::remove_dir_all(root).ok();
}

async fn transfer(
    harness: &DeliveryHarness,
    post: &'static str,
    url: &str,
    network: NetworkClass,
    generation: u64,
) {
    assert!(
        harness
            .handle
            .update_network_status(DeliveryNetworkStatus::new(network, generation)),
        "the network generation must advance"
    );
    let item = sized_item(post, url, 16, 1_000);
    let admission = harness
        .handle
        .update_focus(focus_now(vec![item], 0, generation * 1_000));
    assert_eq!(
        admission,
        FocusAdmission::Accepted,
        "the transfer focus must be admitted"
    );
    wait_for_ranges(&harness.store, post, &[(0, 16)]).await;
}

fn assert_observed(stats: &HostStats, url: &str, network: NetworkClass) {
    let now = wait::now_ms();
    let query = query(url, network, now);
    let estimate = stats
        .origin_model()
        .estimate(&query, now, DecisionMode::Normal);
    assert!(estimate.effective_samples > 0.0, "missing {network:?}");
}

fn assert_unavailable_is_cold(stats: &HostStats, url: &str) {
    let now = wait::now_ms();
    let query = query(url, NetworkClass::Unavailable, now);
    let estimate = stats
        .origin_model()
        .estimate(&query, now, DecisionMode::Normal);
    assert_eq!(
        estimate.effective_samples, 0.0,
        "unclassified network evidence must remain cold"
    );
    assert_eq!(
        estimate.context.network,
        NetworkClass::Unavailable,
        "the estimate must preserve the requested network class"
    );
}

fn query(url: &str, network: NetworkClass, observed_at_ms: u64) -> OriginQuery {
    OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::FullGet, 16, MediaClass::Unknown)
            .with_network(network)
            .with_observed_at_ms(observed_at_ms),
    )
}
