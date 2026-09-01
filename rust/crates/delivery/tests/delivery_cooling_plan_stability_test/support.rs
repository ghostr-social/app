use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_partial_store::partial_range_store::PartialRangeStore;

const WAIT_LIMIT: Duration = Duration::from_secs(30);

pub async fn wait_for_generation(
    handle: &DeliveryHandle,
    notifier: &tokio::sync::Notify,
    wanted: u64,
) {
    tokio::time::timeout(WAIT_LIMIT, async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if handle
                .latest_plan()
                .is_some_and(|plan| plan.focus_generation == Some(wanted))
            {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("duplicate-focus plan was not published");
}

pub async fn wait_for_useful_bytes(store: &PartialRangeStore) {
    let notifier = store.change_notifier();
    tokio::time::timeout(WAIT_LIMIT, async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if !store
                .present_ranges("useful")
                .await
                .expect("stored ranges")
                .is_empty()
            {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("released useful response was not stored");
}
