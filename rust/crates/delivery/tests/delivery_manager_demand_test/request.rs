use super::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use core::future::Future;
use core::time::Duration;
use tokio::time::{interval_at, Instant};

const WAIT_LIMIT: Duration = Duration::from_secs(30);
const KEEP_ALIVE: Duration = Duration::from_secs(5);

pub async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(WAIT_LIMIT, origin.next())
        .await
        .expect("origin request starts")
}

pub async fn keep_alive_until<F>(request: &ActiveRequest, future: F) -> (F::Output, usize)
where
    F: Future,
{
    let mut ticks = interval_at(Instant::now() + KEEP_ALIVE, KEEP_ALIVE);
    let mut sent = 0;
    tokio::pin!(future);
    loop {
        tokio::select! {
            biased;
            output = &mut future => return (output, sent),
            _ = ticks.tick() => {
                assert!(request.send_byte().await, "held request stays open");
                sent += 1;
            }
        }
    }
}

pub async fn finish(request: ActiveRequest, already_sent: usize) {
    let length = usize::try_from(request.range.end - request.range.start).expect("fixture range");
    let remaining = length.checked_sub(already_sent).expect("bounded trickle");
    if remaining > 0 {
        assert!(request.send_bytes(remaining).await, "request stays open");
    }
}
