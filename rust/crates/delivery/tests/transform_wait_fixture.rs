use ghostr_engine::representation::RepresentationBinding;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, StoredMediaSnapshot};
use std::time::Duration;

pub async fn wait_for_transform(
    store: &PartialRangeStore,
    input: &RepresentationBinding,
    handle: &ghostr_delivery::delivery_events::DeliveryHandle,
) -> StoredMediaSnapshot {
    let waiting = async {
        loop {
            let snapshot = store.media_snapshot("post").await.unwrap();
            if snapshot
                .binding()
                .is_some_and(|binding| binding.derives_from(input))
            {
                return snapshot;
            }
            store.change_notifier().notified().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(2), waiting)
        .await
        .unwrap_or_else(|_| panic!("transform timed out: {:?}", handle.decision_history()))
}
