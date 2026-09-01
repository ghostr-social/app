use crate::gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use crate::support::TOTAL;
use std::time::Duration;

const TAIL_START: u64 = 262_144;

pub async fn wait_for_tail(harness: &ProgressiveDeliveryHarness) {
    let observed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let snapshot = harness
                .delivery
                .store
                .media_snapshot("p6")
                .await
                .expect("p6 snapshot");
            if snapshot
                .ranges()
                .iter()
                .any(|range| range.start <= TAIL_START && range.end >= TOTAL)
            {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(observed.is_ok(), "p6 tail never became stable");
}
