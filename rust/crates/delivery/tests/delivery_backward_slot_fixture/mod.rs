use super::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use super::delivery_fixture::DeliveryHarness;
use core::time::Duration;

pub async fn outgoing_body(
    origin: &mut ControlledOrigin,
    harness: &DeliveryHarness,
) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(180), await_outgoing(origin, harness))
        .await
        .expect("live outgoing body setup")
}

async fn await_outgoing(origin: &mut ControlledOrigin, harness: &DeliveryHarness) -> ActiveRequest {
    let notifier = harness.store.change_notifier();
    let mut active = None;
    loop {
        let changed = notifier.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        if body_ready(harness, active.as_ref()).await {
            return active.expect("live body");
        }
        tokio::select! {
            request = origin.next() => {
                assert_eq!(request.path, "/outgoing.mp4", "only the outgoing post is focused during setup");
                request.send_byte().await;
                active = Some(request);
            }
            () = changed => {}
        }
    }
}

async fn body_ready(harness: &DeliveryHarness, request: Option<&ActiveRequest>) -> bool {
    if !request.is_some_and(ActiveRequest::is_open) {
        return false;
    }
    harness
        .store
        .present_ranges("outgoing")
        .await
        .expect("fixture ranges")
        .iter()
        .any(|range| range.contains(&0))
}
