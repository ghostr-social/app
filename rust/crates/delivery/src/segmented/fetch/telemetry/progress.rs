use super::{OriginTelemetry, SegmentedTraffic};
use crate::delivery_events::DeliveryNetworkStatusReader;
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::RequestAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use tokio::time::Instant;

pub(in crate::segmented) struct FetchProgress {
    clock: Mutex<Option<OriginClock>>,
    network_bytes: AtomicU64,
    traffic: Mutex<Option<SegmentedTraffic>>,
}

impl FetchProgress {
    pub(in crate::segmented) fn new(traffic: Option<SegmentedTraffic>) -> Self {
        Self {
            clock: Mutex::new(None),
            network_bytes: AtomicU64::new(0),
            traffic: Mutex::new(traffic),
        }
    }

    pub(in crate::segmented) fn mark_admitted(
        &self,
        requests: &MediaRequestExecutor,
        url: &str,
        network: &DeliveryNetworkStatusReader,
    ) {
        *self.clock() = Some(OriginClock::start(requests, url, network));
    }

    pub(in crate::segmented) fn received(
        &self,
        requests: &MediaRequestExecutor,
        response: &MediaResponse,
    ) {
        let ttfb = self
            .clock()
            .as_mut()
            .expect("admitted HLS request clock")
            .received(requests, response);
        if let Some(traffic) = self.traffic().as_mut() {
            traffic.opened(response, ttfb);
        }
    }

    pub(in crate::segmented) fn add_network_bytes(&self, bytes: u64) {
        self.network_bytes.fetch_add(bytes, Ordering::Relaxed);
        if let Some(traffic) = self.traffic().as_ref() {
            traffic.progress(bytes);
        }
    }

    pub(in crate::segmented) fn origin(&self) -> Option<OriginTelemetry> {
        self.clock().as_ref().map(OriginClock::snapshot)
    }

    pub(in crate::segmented) fn has_admission(&self) -> bool {
        self.clock().is_some()
    }

    pub(in crate::segmented) fn network_bytes(&self) -> u64 {
        self.network_bytes.load(Ordering::Relaxed)
    }

    pub(in crate::segmented) fn close_traffic(&self) {
        self.traffic().take();
    }

    fn clock(&self) -> MutexGuard<'_, Option<OriginClock>> {
        self.clock.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn traffic(&self) -> MutexGuard<'_, Option<SegmentedTraffic>> {
        self.traffic
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

impl Default for FetchProgress {
    fn default() -> Self {
        Self::new(None)
    }
}

struct OriginClock {
    started: Instant,
    redirect_wait: Duration,
    ttfb: Option<Duration>,
    concurrency: usize,
    network_class: NetworkClass,
}

impl OriginClock {
    fn start(
        requests: &MediaRequestExecutor,
        url: &str,
        network: &DeliveryNetworkStatusReader,
    ) -> Self {
        Self {
            started: Instant::now(),
            redirect_wait: Duration::ZERO,
            ttfb: None,
            concurrency: concurrency(requests, url),
            network_class: network.network_class(),
        }
    }

    fn received(&mut self, requests: &MediaRequestExecutor, response: &MediaResponse) -> Duration {
        self.redirect_wait = response.redirect_admission_wait();
        let ttfb = response.origin_elapsed(self.started.elapsed());
        self.ttfb = Some(ttfb);
        self.concurrency = concurrency(requests, response.url().as_str());
        ttfb
    }

    fn snapshot(&self) -> OriginTelemetry {
        OriginTelemetry {
            elapsed: self.started.elapsed().saturating_sub(self.redirect_wait),
            ttfb: self.ttfb,
            concurrency: self.concurrency,
            network_class: self.network_class,
        }
    }
}

fn concurrency(requests: &MediaRequestExecutor, url: &str) -> usize {
    RequestAuthority::from_url(url)
        .map(|authority| requests.active_for(&authority))
        .unwrap_or(1)
        .max(1)
}
