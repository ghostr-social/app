//! A local HEAD usefulness timeout must not poison cross-method host ranking.

mod delivery_fixture;
mod delivery_head_timeout_feedback_origin;

use core::num::NonZeroUsize;
use core::time::Duration;
use delivery_fixture::head_window::serve_visible_current;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::host_stats::{host_of, HostStats};
use serde_json::Value;

#[tokio::test]
async fn head_usefulness_timeout_is_method_specific_timeout_feedback() {
    let origin = delivery_head_timeout_feedback_origin::serve().await;
    let mut options = DeliveryOptions::default();
    options.tuning.max_requests_per_authority = Some(NonZeroUsize::MIN);
    let harness = start_harness("head-timeout-feedback", options);
    let current = serve_visible_current().await;
    harness.handle.update_focus(focus_now(
        vec![current.item(), unsized_item("future", &origin.url)],
        0,
        0,
    ));
    current.assert_get_without_head().await;
    tokio::time::timeout(Duration::from_secs(30), origin.head_started)
        .await
        .expect("HEAD request start")
        .expect("origin start signal");

    let json = wait_for_stats(&harness.root.join("host_stats.json")).await;
    let stats = HostStats::from_json(&json).expect("valid host stats");
    let host = host_of(&origin.url).expect("fixture host");

    let feedback = (stats.failure_ratio(&host), head_error_counts(&json));
    assert_eq!(feedback, (0.0, serde_json::Map::new()));
    assert!(
        !origin_breaker_methods(&json)
            .iter()
            .any(|method| method == "Head"),
        "local HEAD deadline must not affect the HEAD circuit"
    );
    let observed = tokio::time::timeout(Duration::from_secs(2), origin.requests)
        .await
        .expect("HEAD must yield to body")
        .expect("origin task");
    assert!(observed.head.starts_with(b"HEAD "));
    assert!(
        observed.body.starts_with(b"GET "),
        "second request was {}",
        String::from_utf8_lossy(&observed.body)
    );
    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_stats(path: &std::path::Path) -> String {
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if let Ok(json) = tokio::fs::read_to_string(path).await {
                if !json.is_empty() {
                    return json;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("persisted origin feedback")
}

fn head_error_counts(json: &str) -> serde_json::Map<String, Value> {
    let value: Value = serde_json::from_str(json).expect("valid stats JSON");
    let records = value["origin_model"]["global"]
        .as_array()
        .expect("origin records");
    let counts = records
        .iter()
        .find(|record| record[0]["method"] == "Head")
        .into_iter()
        .flat_map(|record| ["long", "short"].map(|period| &record[1][period]))
        .find_map(|period| period["errors"]["counts"].as_object().cloned())
        .unwrap_or_default();
    counts
}

fn origin_breaker_methods(json: &str) -> Vec<String> {
    let value: Value = serde_json::from_str(json).expect("valid stats JSON");
    value["origin_model"]["circuits"]["breakers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|breaker| breaker[0]["method"].as_str().map(str::to_owned))
        .collect()
}
