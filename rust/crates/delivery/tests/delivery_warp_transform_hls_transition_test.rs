mod blocking_transform_fixture;
mod delivery_fixture;
mod focus_wait_fixture;
mod transform_delivery_fixture;

use blocking_transform_fixture::BlockingRemux;
use delivery_fixture::decision::wait_for_history;
use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness_with_store, temp_directory};
use focus_wait_fixture::wait_for_focus;
use ghostr_engine::adaptive::{DecisionOutcome, RecordedWarpCommand};
use ghostr_engine::DeliveryKind;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use transform_delivery_fixture::{report_unsupported, seed_input};

const INPUT: &[u8] = b"ftyp|mdat:frames|moov:index";

#[tokio::test]
async fn progressive_to_hls_cancels_the_obsolete_transform() {
    let root = temp_directory("warp-transform-hls-transition");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let mut item = sized_item(
        "post",
        "https://origin.example/video.mp4",
        INPUT.len() as u64,
        1_000,
    );
    let input = seed_input(&store, &item, INPUT).await;
    let backend = Arc::new(BlockingRemux::new());
    let options = DeliveryOptions {
        transform: Some(backend.clone()),
        ..DeliveryOptions::default()
    };
    let harness = start_harness_with_store(store, root, options);
    harness
        .handle
        .update_focus(focus_now(vec![item.clone()], 0, 0));
    wait_for_focus(&harness.cache).await;
    report_unsupported(&harness.handle, &harness.store, input).await;
    backend.wait_until_entered().await;

    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls = vec![serve(HlsGate::new()).await];
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    wait_for_history(&harness.handle, |history| {
        history.records.iter().any(cancelled_transform)
    })
    .await;
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn cancelled_transform(record: &ghostr_engine::adaptive::DecisionRecord) -> bool {
    matches!(record.eventual_outcome, DecisionOutcome::Cancelled { .. })
        && matches!(
            record
                .warp_decision
                .as_ref()
                .and_then(|warp| warp.selected.as_ref())
                .map(|action| &action.command),
            Some(RecordedWarpCommand::Transform { .. })
        )
}
