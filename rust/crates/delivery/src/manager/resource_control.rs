use crate::manager::traffic::SAMPLE_INTERVAL;
use ghostr_engine::adaptive::{
    ResourceFeedback, ResourceFeedbackCursor, ResourceObservation, ResourcePriceSnapshot,
    ShadowPriceController,
};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::time::Instant;

#[derive(Clone)]
pub(crate) struct ResourceControl {
    state: Arc<Mutex<ResourceState>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResourceEnvironment {
    storage_bytes: u64,
    target: ResourceObservation,
}

impl ResourceEnvironment {
    pub(crate) const fn new(storage_bytes: u64, target: ResourceObservation) -> Self {
        Self {
            storage_bytes,
            target,
        }
    }

    #[cfg(test)]
    pub(crate) const fn target(self) -> ResourceObservation {
        self.target
    }
}

impl ResourceControl {
    pub(crate) fn bootstrap(
        config: &crate::manager::DeliveryManagerConfig,
        origin: Instant,
    ) -> Self {
        let network = bootstrap_network_target(config);
        let cpu = config.transform.as_ref().map_or(0, |backend| {
            let hard = backend.profile().limits().cpu_ms().min(500);
            ShadowPriceController::cpu_operating_target_ms(hard)
        });
        let requests = config.params.concurrency(config.level).max(1) as u64;
        let target = ResourceObservation::new(network, u64::MAX, cpu, requests);
        Self::new(origin, ResourceEnvironment::new(0, target))
    }

    pub(crate) fn new(origin: Instant, environment: ResourceEnvironment) -> Self {
        Self {
            state: Arc::new(Mutex::new(ResourceState::new(origin, environment))),
        }
    }

    pub(crate) fn origin(&self) -> Instant {
        self.lock().origin
    }

    pub(crate) fn record_network_bytes(&self, bytes: u64) {
        let mut state = self.lock();
        state.advance(Instant::now());
        state.network_bytes = state.network_bytes.saturating_add(u128::from(bytes));
    }

    pub(crate) fn record_cpu_ms(&self, cpu_ms: u64) {
        let mut state = self.lock();
        state.advance(Instant::now());
        state.cpu_ms = state.cpu_ms.saturating_add(u128::from(cpu_ms));
    }

    pub(crate) fn record_request(&self) {
        let mut state = self.lock();
        state.advance(Instant::now());
        state.requests = state.requests.saturating_add(1);
    }

    pub(crate) fn feedback(&self, next: ResourceEnvironment) -> ResourceFeedback {
        let mut state = self.lock();
        state.advance(Instant::now());
        state.environment = next;
        state.feedback
    }

    fn lock(&self) -> MutexGuard<'_, ResourceState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}

struct ResourceState {
    origin: Instant,
    tick: u128,
    environment: ResourceEnvironment,
    network_bytes: u128,
    cpu_ms: u128,
    requests: u128,
    prices: ShadowPriceController,
    feedback: ResourceFeedback,
}

impl ResourceState {
    fn new(origin: Instant, environment: ResourceEnvironment) -> Self {
        let cursor = ResourceFeedbackCursor::new(0, 0);
        let snapshot = ResourcePriceSnapshot::new(cursor, Default::default());
        Self {
            origin,
            tick: 0,
            environment,
            network_bytes: 0,
            cpu_ms: 0,
            requests: 0,
            prices: ShadowPriceController::default(),
            feedback: ResourceFeedback::authoritative(
                snapshot,
                ResourceObservation::default(),
                environment.target,
            ),
        }
    }

    fn advance(&mut self, now: Instant) {
        let next = now
            .saturating_duration_since(self.origin)
            .as_nanos()
            .saturating_div(SAMPLE_INTERVAL.as_nanos());
        if next <= self.tick {
            return;
        }
        let count = next - self.tick;
        self.close_current(self.tick + 1);
        self.close_empty(count - 1, next);
        self.clear_usage();
        self.tick = next;
    }

    fn close_current(&mut self, tick: u128) {
        let target = self.environment.target;
        let actual = ResourceObservation::new(
            rate(self.network_bytes),
            self.environment.storage_bytes,
            clamp(self.cpu_ms),
            clamp(self.requests),
        );
        self.prices.observe(actual, target);
        self.publish(tick, actual, target);
    }

    fn close_empty(&mut self, count: u128, tick: u128) {
        if count == 0 {
            return;
        }
        let target = self.environment.target;
        let actual = ResourceObservation::new(0, self.environment.storage_bytes, 0, 0);
        self.prices.observe_repeated(actual, target, count);
        self.publish(tick, actual, target);
    }

    fn publish(&mut self, tick: u128, actual: ResourceObservation, target: ResourceObservation) {
        let snapshot = ResourcePriceSnapshot::new(cursor(tick), self.prices.prices());
        self.feedback = ResourceFeedback::authoritative(snapshot, actual, target);
    }

    fn clear_usage(&mut self) {
        self.network_bytes = 0;
        self.cpu_ms = 0;
        self.requests = 0;
    }
}

impl ghostr_net::media_request_executor::MediaResourceObserver for ResourceControl {
    fn record_request(&self) {
        ResourceControl::record_request(self);
    }

    fn record_response_bytes(&self, bytes: u64) {
        self.record_network_bytes(bytes);
    }
}

fn rate(bytes: u128) -> u64 {
    let per_second = bytes
        .saturating_mul(1_000_000_000)
        .saturating_div(SAMPLE_INTERVAL.as_nanos());
    clamp(per_second)
}

fn clamp(value: u128) -> u64 {
    value.min(u128::from(u64::MAX)) as u64
}

pub(crate) const fn cursor(tick: u128) -> ResourceFeedbackCursor {
    ResourceFeedbackCursor::new((tick >> 64) as u64, tick as u64)
}

fn bootstrap_network_target(config: &crate::manager::DeliveryManagerConfig) -> u64 {
    let configured = config.network.profile().bandwidth_kbps.saturating_mul(125);
    match configured {
        0 => ghostr_engine::host_stats::OPTIMISTIC_THROUGHPUT_BPS as u64,
        value => value,
    }
}
