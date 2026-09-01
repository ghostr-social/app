use super::Scenario;
use crate::delivery_fixture::full_recovery_origin::{ObservedRequest, REQUEST_TIMEOUT};
use axum::http::Method;
use core::time::Duration;

mod range;
use range::trial_slice_bytes;

const MAX_SUPERSEDED_REQUESTS: usize = 8;
const QUIET_PERIOD: Duration = Duration::from_millis(50);

impl Scenario {
    pub(super) async fn next_trial_request(&mut self) -> ObservedRequest {
        let waiting = async {
            let mut trial_slice_seen = false;
            for skipped in 0..=MAX_SUPERSEDED_REQUESTS {
                let request = self.origin.next().await;
                if is_trial(&request) {
                    return request;
                }
                assert!(
                    skipped < MAX_SUPERSEDED_REQUESTS,
                    "too many superseded requests before recovery trial"
                );
                settle_before_trial(request, &mut trial_slice_seen).await;
            }
            unreachable!("bounded request loop returns or panics");
        };
        tokio::time::timeout(REQUEST_TIMEOUT, waiting)
            .await
            .expect("missing Full recovery trial")
    }

    pub(super) async fn assert_no_competing_trial(&mut self) {
        let waiting = async {
            for observed in 0..=MAX_SUPERSEDED_REQUESTS {
                match tokio::time::timeout(QUIET_PERIOD, self.origin.next()).await {
                    Ok(request) if observed < MAX_SUPERSEDED_REQUESTS => {
                        settle_superseded(request).await;
                    }
                    Ok(_) => panic!("unbounded requests while recovery trial is open"),
                    Err(_) => return,
                }
            }
            unreachable!("bounded quiet loop returns or panics");
        };
        tokio::time::timeout(REQUEST_TIMEOUT, waiting)
            .await
            .expect("superseded request did not cancel");
    }
}

async fn settle_before_trial(request: ObservedRequest, trial_slice_seen: &mut bool) {
    if let Some(bytes) = trial_slice_bytes(&request) {
        assert!(!*trial_slice_seen, "duplicate trial bootstrap range");
        *trial_slice_seen = true;
        request.finish(bytes).await;
        return;
    }
    settle_superseded(request).await;
}

async fn settle_superseded(request: ObservedRequest) {
    if is_superseded_head(&request) {
        return;
    }
    if is_superseded_range(&request) {
        request.await_cancellation().await;
        return;
    }
    panic!(
        "unexpected {} {} range={:?}",
        request.method, request.path, request.range
    );
}

fn is_trial(request: &ObservedRequest) -> bool {
    request.method == Method::GET && request.path == "/trial.mp4" && request.range.is_none()
}

fn is_superseded_head(request: &ObservedRequest) -> bool {
    request.method == Method::HEAD && request.range.is_none() && is_superseded_path(request)
}

fn is_superseded_range(request: &ObservedRequest) -> bool {
    if request.method != Method::GET {
        return false;
    }
    matches!(
        (request.path.as_str(), request.range.as_deref()),
        ("/probe.mp4", Some("bytes=0-65535")) | ("/parallel.mp4", Some("bytes=0-4095"))
    )
}

fn is_superseded_path(request: &ObservedRequest) -> bool {
    matches!(request.path.as_str(), "/probe.mp4" | "/parallel.mp4")
}
