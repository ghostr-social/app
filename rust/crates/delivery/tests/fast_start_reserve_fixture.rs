use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::adaptive::ReserveCandidateState;
use ghostr_engine::PostId;

pub async fn wait_for_state(
    handle: &DeliveryHandle,
    post: &PostId,
    after: u64,
    ready: bool,
) -> u64 {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(revision) = matching_revision(handle, post, after, ready) {
                return revision;
            }
            changed.await;
        }
    })
    .await
    .expect("fresh exact reserve state")
}

fn matching_revision(
    handle: &DeliveryHandle,
    post: &PostId,
    after: u64,
    ready: bool,
) -> Option<u64> {
    handle.plan_history().into_iter().find_map(|entry| {
        let state = entry
            .plan
            .ready_reserve
            .candidates
            .iter()
            .find(|item| &item.post == post)
            .map(|item| &item.state)?;
        let matches = match state {
            ReserveCandidateState::Ready { .. } => ready,
            ReserveCandidateState::Structural { .. } => !ready,
            _ => false,
        };
        (entry.revision > after && matches).then_some(entry.revision)
    })
}
