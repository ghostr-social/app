//! Storage-dependent work cannot launch when its planned eviction was not applied.

mod delivery_fixture;

use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::full_disk::{discard, limits, spaced_store};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_with_store;
use delivery_fixture::wait::wait_for_file_with_limit;
use std::sync::Arc;
use std::time::Duration;

const UNREACHABLE: &str = "http://127.0.0.1:9/video.mp4";

#[tokio::test]
async fn missed_policy_eviction_never_launches_its_dependent_allocation() {
    let fixture = spaced_store("ghostr-eviction-apply-fence", limits(100, 0), 10_000);
    let items: Vec<_> = (0..9)
        .map(|index| {
            sized_item(
                Box::leak(format!("p{index}").into_boxed_str()),
                UNREACHABLE,
                100,
                1_000,
            )
        })
        .collect();
    seed_range(&fixture.store, &items[1], 0, &[1; 45]).await;
    seed_range(&fixture.store, &items[8], 0, &[8; 55]).await;
    let _blocked_victim = fixture.store.lease("p8");
    let root = fixture.root.clone();
    let harness = start_harness_with_store(
        Arc::new(fixture.store),
        root.clone(),
        DeliveryOptions::default(),
    );

    harness.handle.update_focus(focus_now(items, 0, 0));
    // The single actor consumes this zero-debounce save only after its rejected reconcile ends.
    wait_for_file_with_limit(&root.join("qoe_stats.json"), Duration::from_secs(30)).await;

    assert_eq!(harness.store.used_bytes().await, 100);
    let decisions = harness.handle.decision_history();
    assert!(
        decisions.records.is_empty(),
        "a rejected planning transaction published {decisions:#?}"
    );
    assert!(harness.handle.plan_history().is_empty());
    assert_eq!(
        harness.handle.evaluation_snapshot().budget.observations,
        0,
        "a rejected plan must not corrupt applied-plan metrics"
    );
    let evidence = harness.handle.evidence_page_json(0, 0).expect("evidence");
    let evidence: serde_json::Value = serde_json::from_str(&evidence).expect("evidence schema");
    assert!(
        evidence["evaluation"]["efficiency"]["cpu_micros"]
            .as_u64()
            .is_some_and(|value| value > 0),
        "a rejected plan must retain its measured planner cost"
    );
    assert_eq!(
        harness
            .store
            .present_ranges("p1")
            .await
            .expect("valid test fixture"),
        vec![0..45]
    );
    discard(&root);
}
