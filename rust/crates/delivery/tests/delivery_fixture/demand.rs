use super::DeliveryHarness;
use core::time::Duration;
use ghostr_delivery::playback_demand::DemandConsumer;
use ghostr_engine::{ByteRange, PostId};

pub async fn blocked(harness: &DeliveryHarness, post: &str, range: ByteRange) -> DemandConsumer {
    let binding = wait_for_binding(harness, post).await;
    let mut consumer = harness.demand.consumer(PostId::new(post), Some(binding));
    consumer.demand(range);
    consumer
}

pub async fn wait_for_binding(
    harness: &DeliveryHarness,
    post: &str,
) -> ghostr_engine::representation::RepresentationBinding {
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
    binding
}
