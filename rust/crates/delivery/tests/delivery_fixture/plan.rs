//! Event-driven waits for published delivery plans.

use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryHandle;

pub async fn wait_for_current(handle: &DeliveryHandle, expected: &str) {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if handle
                .latest_plan()
                .and_then(|plan| plan.current)
                .is_some_and(|post| post.as_str() == expected)
            {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("focused plan publication");
}
