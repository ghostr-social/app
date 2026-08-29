//! Load-tolerant waits for positive HLS fixture starts.

use super::evidence::DeliveryEvidence as _;
use super::hls::HlsGate;
use super::DeliveryHarness;
use core::fmt::Debug;
use core::time::Duration;
use tokio::time::{timeout_at, Instant};

mod trace;

use trace::WaitTrace;

const WAIT_LIMIT: Duration = Duration::from_secs(30);
const DECISION_LIMIT: usize = 8;

pub async fn wait_for_start(harness: &DeliveryHarness, gate: &HlsGate, post: &str, context: &str) {
    wait_for_starts(harness, gate, &[post], context).await;
}

pub async fn wait_for_starts(
    harness: &DeliveryHarness,
    gate: &HlsGate,
    posts: &[&str],
    context: &str,
) {
    StartExpectation {
        harness,
        gate,
        posts,
        context,
    }
    .wait()
    .await;
}

struct StartExpectation<'a> {
    harness: &'a DeliveryHarness,
    gate: &'a HlsGate,
    posts: &'a [&'a str],
    context: &'a str,
}

impl StartExpectation<'_> {
    async fn wait(&self) {
        let trace = WaitTrace::new(self.posts.len(), WAIT_LIMIT);
        let acquired = self.gate.started.acquire_many(trace.expected);
        match timeout_at(trace.deadline, acquired).await {
            Ok(Ok(permit)) => permit.forget(),
            Ok(Err(error)) => self.fail(&trace, error),
            Err(_) => self.boundary(&trace),
        }
    }

    fn boundary(&self, trace: &WaitTrace) {
        match self.gate.started.try_acquire_many(trace.expected) {
            Ok(permit) => permit.forget(),
            Err(error) => self.fail(trace, error),
        }
    }

    fn fail(&self, trace: &WaitTrace, error: impl Debug) -> ! {
        let snapshots: Vec<_> = self
            .posts
            .iter()
            .map(|post| (*post, self.harness.segmented.snapshot(post)))
            .collect();
        let mut decisions = self.harness.handle.decision_history().records;
        decisions = decisions.into_iter().rev().take(DECISION_LIMIT).collect();
        panic!(
            "{}: error={error:?}; elapsed={:?}; wall={:?}; overshoot={:?}; expected={}; \
             blocked={}; hits={:?}; started={}; released={}; closed={}; snapshots={snapshots:?}; \
             plan={:?}; recent_decisions={decisions:?}",
            self.context,
            trace.started.elapsed(),
            trace.wall_started.elapsed(),
            Instant::now().saturating_duration_since(trace.deadline),
            trace.expected,
            self.gate.blocked(),
            self.gate.hits(),
            self.gate.started.available_permits(),
            self.gate.release.available_permits(),
            self.gate.started.is_closed(),
            self.harness.handle.latest_plan(),
        )
    }
}
