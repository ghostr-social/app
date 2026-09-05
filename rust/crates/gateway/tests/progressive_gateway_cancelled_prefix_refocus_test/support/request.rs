use crate::gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use crate::support::{ActiveRequest, ControlledOrigin};
use core::time::Duration;

pub async fn held_prefix(origin: &mut ControlledOrigin) -> ActiveRequest {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let request = next_until(origin, deadline, "initial next-item prefix").await;
        if is_prefix(&request) {
            return request;
        }
        complete(&request).await;
    }
}

pub async fn next_prefix(
    origin: &mut ControlledOrigin,
    harness: &ProgressiveDeliveryHarness,
) -> ActiveRequest {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let request = tokio::time::timeout_at(deadline, origin.next())
            .await
            .unwrap_or_else(|_| replacement_timeout(harness));
        if is_prefix(&request) {
            return request;
        }
        complete(&request).await;
    }
}

fn replacement_timeout(harness: &ProgressiveDeliveryHarness) -> ! {
    panic!(
        "replacement p6 prefix did not start after first active demand; demands={:#?}; \
         latest_plan={:#?}",
        harness.delivery.demands(),
        harness.delivery.handle.latest_plan(),
    )
}

async fn next_until(
    origin: &mut ControlledOrigin,
    deadline: tokio::time::Instant,
    label: &str,
) -> ActiveRequest {
    tokio::time::timeout_at(deadline, origin.next())
        .await
        .unwrap_or_else(|_| panic!("{label} request starts"))
}

fn is_prefix(request: &ActiveRequest) -> bool {
    request.path == "/p6.mp4" && request.range.start == 0
}

async fn complete(request: &ActiveRequest) {
    let length = request.range.end.saturating_sub(request.range.start) as usize;
    let _ = request.send_bytes(length).await;
}

pub async fn wait_closed(request: &ActiveRequest) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while request.is_open() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("zero-byte prefix cancellation closes origin body");
}
