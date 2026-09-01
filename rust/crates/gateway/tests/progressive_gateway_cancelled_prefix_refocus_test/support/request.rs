use crate::gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use crate::support::{ActiveRequest, ControlledOrigin};
use std::time::Duration;

const TAIL_START: u64 = 262_144;

pub async fn held_prefix_after_tail(origin: &mut ControlledOrigin) -> ActiveRequest {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut prefix = None;
    let mut tail_done = false;
    loop {
        let request = next_until(origin, deadline, "initial prefix and tail").await;
        if is_prefix(&request) {
            if tail_done {
                return request;
            }
            prefix = Some(request);
            continue;
        }
        let is_tail = request.path == "/p6.mp4" && request.range.start == TAIL_START;
        complete(&request).await;
        if is_tail {
            if let Some(prefix) = prefix {
                return prefix;
            }
            tail_done = true;
        }
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
