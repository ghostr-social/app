mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_delivery::delivery_events::{
    DeliveryFocus, DeliveryNetworkStatus, FocusAdmission, FocusGeneration, FocusItem,
    FocusTransition,
};
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn plan_records_the_applied_focus_lineage_and_network_generation() {
    let harness = start_harness("ghostr-plan-causal-generation", DeliveryOptions::default());
    assert_eq!(
        harness.handle.update_focus(focus(7)),
        FocusAdmission::Accepted
    );
    assert_eq!(
        harness.handle.update_focus(focus(8)),
        FocusAdmission::Accepted
    );
    assert!(harness
        .handle
        .update_network_status(DeliveryNetworkStatus::new(NetworkClass::Constrained, 41,)));
    let network = harness
        .handle
        .update_network_profile(lossy_profile())
        .expect("network command");

    let plan = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(plan) = harness.handle.latest_plan().filter(|plan| {
                plan.focus_generation == Some(8)
                    && plan.network_status_generation == 41
                    && plan.network_profile_generation == network
            }) {
                return plan;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("causally fenced plan");

    assert_eq!(plan.current.as_ref().map(PostId::as_str), Some("current"));
    assert_eq!(plan.focus_covers_from, Some(7));
    assert_eq!(plan.network_class, NetworkClass::Constrained);
    assert_eq!(harness.network.profile(), lossy_profile());
}

fn focus(generation: u64) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("current"),
            meta: VideoMeta {
                urls: vec!["https://media.example/current.mp4".into()],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(16),
                duration_ms: Some(1_000),
            },
        }],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).expect("valid test fixture"),
        transition: FocusTransition::UserNavigation,
        rescue: None,
    }
}

fn lossy_profile() -> NetworkProfile {
    NetworkProfile {
        bandwidth_kbps: 700,
        latency_ms: 50,
        packet_loss_bps: 6_000,
        max_connections_per_host: 1,
    }
}
