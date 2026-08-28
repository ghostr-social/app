mod delivery_fixture;
mod fast_start_manager_fixture;
mod fast_start_mp4_fixture;
mod fast_start_player_fixture;
mod fast_start_reserve_fixture;
mod focus_wait_fixture;
mod transform_wait_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness_with_store, temp_directory};
use fast_start_manager_fixture::{authority, seed};
use fast_start_mp4_fixture::{front_indexed_mp4, tail_indexed_mp4};
use fast_start_player_fixture::{report_failed, report_ready};
use fast_start_reserve_fixture::wait_for_state;
use focus_wait_fixture::wait_for_focus;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_delivery::transform::FastStartRemuxBackend;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use transform_wait_fixture::wait_for_transform;

#[tokio::test]
async fn derived_fast_start_candidate_waits_for_fresh_exact_player_success() {
    let tail = tail_indexed_mp4();
    let front = front_indexed_mp4();
    let root = temp_directory("warp-fast-start-revalidation");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let current = sized_item(
        "current",
        "https://origin.example/current.mp4",
        front.len() as u64,
        1_000,
    );
    let target = sized_item(
        "post",
        "https://origin.example/video.mp4",
        tail.len() as u64,
        1_000,
    );
    seed(&store, &current, &front).await;
    seed(&store, &target, &tail).await;
    let options = DeliveryOptions {
        transform: Some(Arc::new(FastStartRemuxBackend::production())),
        ..DeliveryOptions::default()
    };
    let harness = start_harness_with_store(std::sync::Arc::clone(&store), root, options);
    harness
        .handle
        .update_focus(focus_now(vec![current, target], 0, 0));
    wait_for_focus(&harness.cache).await;
    let current_snapshot = store
        .media_snapshot("current")
        .await
        .expect("valid test fixture");
    let input_snapshot = store
        .media_snapshot("post")
        .await
        .expect("valid test fixture");
    let current_binding = current_snapshot
        .binding()
        .expect("valid test fixture")
        .clone();
    let input = input_snapshot
        .binding()
        .expect("valid test fixture")
        .clone();
    let current_epoch = current_snapshot.content_epoch();
    let input_epoch = input_snapshot.content_epoch();
    report_ready(
        &harness.handle,
        authority(current_binding, current_epoch),
        1,
    )
    .await;
    report_failed(&harness.handle, authority(input.clone(), input_epoch), 1).await;

    let transformed = wait_for_transform(&store, &input, &harness.handle).await;
    let post = PostId::new("post");
    let structural = wait_for_state(&harness.handle, &post, 0, false).await;
    report_ready(&harness.handle, authority(input, input_epoch), 2).await;
    let _ = harness
        .handle
        .update_network_profile(NetworkProfile::default());
    let after_stale = wait_for_state(&harness.handle, &post, structural, false).await;
    let derived = transformed.binding().expect("valid test fixture").clone();
    report_ready(
        &harness.handle,
        authority(derived, transformed.content_epoch()),
        3,
    )
    .await;
    wait_for_state(&harness.handle, &post, after_stale, true).await;
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
