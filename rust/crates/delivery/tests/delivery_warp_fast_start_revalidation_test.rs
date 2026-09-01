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
use fast_start_manager_fixture::{authority, bindings, new_store, seed};
use fast_start_mp4_fixture::{front_indexed_mp4, tail_indexed_mp4};
use fast_start_player_fixture::{report_failed, report_ready, report_rejected};
use fast_start_reserve_fixture::wait_for_state;
use focus_wait_fixture::wait_for_focus;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_delivery::transform::FastStartRemuxBackend;
use ghostr_engine::PostId;
use std::sync::Arc;
use transform_wait_fixture::wait_for_transform;

#[tokio::test]
async fn derived_fast_start_candidate_waits_for_fresh_exact_player_success() {
    let (tail, front) = (tail_indexed_mp4(), front_indexed_mp4());
    let root = temp_directory("warp-fast-start-revalidation");
    let store = new_store(&root);
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
    let [current_binding, target_binding] = bindings([&current, &target]);
    seed(&store, current_binding, &front).await;
    seed(&store, target_binding, &tail).await;
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
    let current_revision = current_snapshot.revision();
    let input_revision = input_snapshot.revision();
    report_ready(
        &harness.handle,
        authority(current_binding, current_revision),
        1,
    )
    .await;
    let baseline = wait_for_state(&harness.handle, &PostId::new("post"), 0, false).await;
    report_failed(&harness.handle, authority(input.clone(), input_revision), 1).await;

    let transformed = wait_for_transform(&store, &input, &harness.handle).await;
    let post = PostId::new("post");
    let structural = wait_for_state(&harness.handle, &post, baseline, false).await;
    report_rejected(&harness.handle, authority(input, input_revision), 2).await;
    let _ = harness
        .handle
        .update_network_profile(NetworkProfile::default());
    let after_stale = wait_for_state(&harness.handle, &post, structural, false).await;
    let derived = transformed.binding().expect("valid test fixture").clone();
    report_ready(
        &harness.handle,
        authority(derived, transformed.revision()),
        3,
    )
    .await;
    wait_for_state(&harness.handle, &post, after_stale, true).await;
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
