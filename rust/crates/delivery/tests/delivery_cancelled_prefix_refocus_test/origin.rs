use super::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use super::PREFIX;
use core::time::Duration;
use tokio::time::Instant;

pub(super) async fn next_prefix(origin: &mut ControlledOrigin) -> ActiveRequest {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut unrelated = Vec::new();
    loop {
        let request = tokio::time::timeout_at(deadline, origin.next())
            .await
            .unwrap_or_else(|_| {
                panic!("replacement p6 prefix request starts; observed={unrelated:?}")
            });
        if covers_prefix(&request) {
            return request;
        }
        unrelated.push(complete(request).await);
    }
}

fn covers_prefix(request: &ActiveRequest) -> bool {
    request.path == "/p6.mp4"
        && request.range.start == PREFIX.start
        && request.range.end >= PREFIX.end
}

async fn complete(request: ActiveRequest) -> (String, core::ops::Range<u64>, bool) {
    let summary = (request.path.clone(), request.range.clone());
    let bytes = request.range.end.saturating_sub(request.range.start);
    let accepted = request.send_bytes(bytes as usize).await;
    (summary.0, summary.1, accepted)
}
