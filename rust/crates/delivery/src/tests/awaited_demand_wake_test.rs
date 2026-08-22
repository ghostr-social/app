use crate::delivery_events::command_channel;
use crate::manager::response_open;
use crate::manager::timeline::TimelineCoordinator;
use crate::manager::transfers::InternalEvent;
use crate::manager::wake::Wake;
use crate::manager::wake_lane::{WakeCursor, WakeLane};
use crate::manager::wake_select::wait_for_channel_wake;
use crate::playback_demand::{demand_channel, ConsumerId, DemandLease, DemandState};
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test(flavor = "current_thread")]
async fn awaited_demand_is_delivered_and_advances_lane_fairness() {
    let (_handle, mut commands) = command_channel();
    let (demand_sender, mut demand) = demand_channel();
    let (_events_sender, mut events) = mpsc::unbounded_channel::<InternalEvent>();
    let (_responses_sender, mut responses) =
        response_open::channel(std::time::Duration::from_secs(1));
    let mut cursor = WakeCursor::default();
    let root = crate::tests::support::temp_directory("awaited-demand-wake");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(tokio::sync::Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let mut timelines = TimelineCoordinator::new(store);
    let signal = DemandState::Blocked(DemandLease::new(
        ConsumerId::new(1).unwrap(),
        PostId::new("playing"),
        None,
        ByteRange::new(4, 8),
    ));
    let delayed = signal.clone();
    tokio::spawn(async move {
        tokio::task::yield_now().await;
        demand_sender.emit(delayed);
    });

    let wake = wait_for_channel_wake(
        &mut commands,
        &mut demand,
        &mut responses,
        &mut events,
        &mut timelines,
        &mut cursor,
    )
    .await
    .expect("demand wake");

    assert!(matches!(wake, Wake::Demand(actual) if actual == signal));
    assert_eq!(
        cursor.choose(&[false, false, false, false, false, false, true, false]),
        Some(WakeLane::Internal)
    );
    tokio::fs::remove_dir_all(root).await.unwrap();
}
