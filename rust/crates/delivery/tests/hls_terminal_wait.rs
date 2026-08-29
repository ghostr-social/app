use core::time::Duration;
use ghostr_delivery::segmented::{SegmentedCache, SegmentedPhase, SegmentedSnapshot};
use std::time::SystemTime;
use tokio::time::{timeout_at, Instant};

const WAIT_LIMIT: Duration = Duration::from_secs(30);
const HISTORY_LIMIT: usize = 32;

pub async fn wait_terminal(cache: &SegmentedCache, post: &str) -> SegmentedSnapshot {
    let changed = cache.notifier();
    let mut trace = WaitTrace::new();
    loop {
        let notified = changed.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        let snapshot = cache.snapshot(post);
        trace.record(&snapshot);
        if terminal(&snapshot) {
            return snapshot;
        }
        if trace.expired() {
            return boundary(cache, post, &mut trace);
        }
        if timeout_at(trace.deadline, notified).await.is_err() {
            return boundary(cache, post, &mut trace);
        }
    }
}

fn boundary(cache: &SegmentedCache, post: &str, trace: &mut WaitTrace) -> SegmentedSnapshot {
    let latest = cache.snapshot(post);
    trace.record(&latest);
    if terminal(&latest) {
        return latest;
    }
    trace.fail(latest)
}

struct WaitTrace {
    started: Instant,
    deadline: Instant,
    wall_started: SystemTime,
    last_change: Instant,
    snapshots: Vec<SegmentedSnapshot>,
}

impl WaitTrace {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            deadline: now + WAIT_LIMIT,
            wall_started: SystemTime::now(),
            last_change: now,
            snapshots: Vec::new(),
        }
    }

    fn record(&mut self, snapshot: &SegmentedSnapshot) {
        if self.snapshots.last() == Some(snapshot) {
            return;
        }
        self.last_change = Instant::now();
        if self.snapshots.len() == HISTORY_LIMIT {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot.clone());
    }

    fn expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    fn fail(&self, snapshot: SegmentedSnapshot) -> ! {
        panic!(
            "terminal HLS readiness: monotonic={:?}; wall={:?}; idle={:?}; \
             snapshots={:?}; snapshot={snapshot:?}",
            self.started.elapsed(),
            self.wall_started.elapsed(),
            self.last_change.elapsed(),
            self.snapshots,
        )
    }
}

fn terminal(snapshot: &SegmentedSnapshot) -> bool {
    matches!(
        snapshot.phase,
        SegmentedPhase::Ready | SegmentedPhase::Failed
    )
}
