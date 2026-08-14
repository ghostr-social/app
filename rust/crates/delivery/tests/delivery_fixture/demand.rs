use super::DeliveryHarness;
use ghostr_delivery::playback_demand::DemandConsumer;
use ghostr_engine::{ByteRange, PostId};
use std::time::Duration;

pub async fn blocked(harness: &DeliveryHarness, post: &str, range: ByteRange) -> DemandConsumer {
    let binding = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(binding) = harness.store.representation_binding(post).await {
                return binding;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("representation binding");
    let mut consumer = harness.demand.consumer(PostId::new(post), Some(binding));
    consumer.demand(range);
    consumer
}
