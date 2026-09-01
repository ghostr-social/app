mod delivery_fixture;
#[path = "delivery_focus_rescue_rollup_test/evidence.rs"]
mod evidence;

use delivery_fixture::items::{seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{
    DeliveryFocus, FocusGeneration, FocusTransition, TransportRescue, TransportRescueReason,
};

const RESCUE_COUNT: u64 = 16;

#[tokio::test(flavor = "current_thread")]
async fn coalesced_rescue_burst_preserves_exact_reason_and_payload_totals() {
    let harness = start_harness("ghostr-focus-rescue-rollup", DeliveryOptions::default());
    let item = sized_item("focused", "http://127.0.0.1:9/focused.mp4", 16, 1_000);
    seed_range(&harness.store, &item, 0, &[7; 16]).await;

    for generation in 1..=RESCUE_COUNT {
        let reason = reasons()[generation as usize % reasons().len()];
        let focus = rescue_focus(item.clone(), generation, reason);
        harness.handle.update_focus(focus);
    }

    let expected_total = RESCUE_COUNT * (RESCUE_COUNT + 1) / 2;
    let observed = evidence::wait_for_rollup(&harness.root, RESCUE_COUNT).await;
    assert_eq!(observed.rank_displacement_total, expected_total);
    assert_eq!(observed.rescue_wait_total_ms, expected_total * 10);
    assert_eq!(observed.reason_counts(), [4; 4]);

    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn rescue_focus(
    item: ghostr_delivery::delivery_events::FocusItem,
    generation: u64,
    reason: TransportRescueReason,
) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![item],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).expect("positive generation"),
        transition: FocusTransition::TransportRescue,
        rescue: Some(TransportRescue {
            reason,
            rank_displacement: generation as u32,
            wait_ms: generation * 10,
        }),
    }
}

fn reasons() -> [TransportRescueReason; 4] {
    [
        TransportRescueReason::EtaUnavailable,
        TransportRescueReason::EtaTooLong,
        TransportRescueReason::DeliveryFailed,
        TransportRescueReason::GraceExpired,
    ]
}
