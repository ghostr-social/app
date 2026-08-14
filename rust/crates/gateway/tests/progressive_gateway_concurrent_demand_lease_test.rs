mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::playback_demand::DemandState;
use std::collections::HashSet;
use tower::ServiceExt;

#[tokio::test]
async fn current_and_prepared_next_responses_own_distinct_demand_leases() {
    let mut harness = progressive_harness("ghostr-progressive-concurrent-demand");
    for post in ["current", "next"] {
        harness.posts.insert(post);
        harness.store.set_total_len(post, 10).await.unwrap();
        harness.store.write_range(post, 0, &[7]).await.unwrap();
    }
    let current = harness.video_request("current", Some("bytes=0-9")).await;
    let next = harness.video_request("next", Some("bytes=0-9")).await;
    let current = harness.router.clone().oneshot(current).await.unwrap();
    let next = harness.router.clone().oneshot(next).await.unwrap();

    let first = blocked(harness.demand.recv().await.expect("first lease"));
    let second = blocked(harness.demand.recv().await.expect("second lease"));

    assert_ne!(first.consumer(), second.consumer());
    assert_eq!(
        HashSet::from([first.post().as_str(), second.post().as_str()]),
        HashSet::from(["current", "next"])
    );
    drop((current, next));
    std::fs::remove_dir_all(harness.root).ok();
}

fn blocked(state: DemandState) -> ghostr_delivery::playback_demand::DemandLease {
    match state {
        DemandState::Blocked(lease) => lease,
        other => panic!("expected blocked lease, got {other:?}"),
    }
}
