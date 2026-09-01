use super::{history, ActiveRequest, ControlledOrigin, DeliveryHandle, Duration};

pub(crate) async fn next_request_while_streaming(
    origin: &mut ControlledOrigin,
    active: &ActiveRequest,
    handle: &DeliveryHandle,
) -> ActiveRequest {
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        let mut tick = tokio::time::interval(Duration::from_millis(50));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            tokio::select! {
                biased;
                request = origin.next() => return request,
                sent = active.send_byte() => assert!(sent, "first range remains active"),
            }
        }
    })
    .await;
    result.unwrap_or_else(|_| {
        panic!(
            "parallel streaming trial timed out; decisions={}",
            history(handle)
        )
    })
}
