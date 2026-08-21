//! Runtime network conditions controlled by the loopback debug page.
//! Defaults are inert, so production delivery is unchanged until a
//! developer explicitly enables simulation.

use ghostr_engine::RequestAuthority;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::Notify;

mod bandwidth;
use bandwidth::SharedBandwidth;

const INVALID_AUTHORITY: &str = "invalid-request-authority";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct NetworkProfile {
    /// Aggregate limit shared by all simulated media transfers.
    /// Zero disables bandwidth pacing.
    pub bandwidth_kbps: u64,
    /// Delay before each media request. Zero disables added latency.
    pub latency_ms: u64,
    /// Independently observed or simulated packet loss in basis points.
    #[serde(default)]
    pub packet_loss_bps: u16,
    /// Simultaneous requests allowed for one host. Zero is unlimited.
    pub max_connections_per_host: usize,
}

#[derive(Clone, Debug)]
pub struct NetworkThrottle {
    inner: Arc<ThrottleInner>,
}

#[derive(Debug, Default)]
struct ThrottleInner {
    profile: RwLock<NetworkProfile>,
    active: Mutex<HashMap<String, usize>>,
    bandwidth: SharedBandwidth,
    profile_changed: Notify,
}

impl Default for NetworkThrottle {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkThrottle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ThrottleInner::default()),
        }
    }

    pub fn profile(&self) -> NetworkProfile {
        *self
            .inner
            .profile
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn update(&self, profile: NetworkProfile) {
        *self
            .inner
            .profile
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = profile;
        self.inner.profile_changed.notify_waiters();
    }

    pub fn active_connections(&self) -> Vec<(String, usize)> {
        let active = self
            .inner
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut hosts: Vec<_> = active
            .iter()
            .map(|(host, count)| (host.clone(), *count))
            .collect();
        hosts.sort_by(|left, right| left.0.cmp(&right.0));
        hosts
    }

    pub async fn acquire(&self, url: &str) -> ConnectionPermit {
        let host = connection_key(url);
        self.claim(&host);
        ConnectionPermit {
            throttle: self.clone(),
            host,
        }
    }

    pub async fn wait_for_latency(&self) {
        let delay = Duration::from_millis(self.profile().latency_ms);
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }

    pub async fn pace(&self, bytes: u64) {
        self.inner
            .bandwidth
            .pace(bytes, &self.inner.profile, &self.inner.profile_changed)
            .await;
    }

    fn claim(&self, host: &str) {
        let mut active = self
            .inner
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *active.entry(host.to_owned()).or_default() += 1;
    }

    fn release(&self, host: &str) {
        let mut active = self
            .inner
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(count) = active.get_mut(host) {
            *count = count.saturating_sub(1);
        }
        active.retain(|_, count| *count > 0);
    }
}

fn connection_key(url: &str) -> String {
    RequestAuthority::from_url(url)
        .map(|authority| authority.as_str().to_owned())
        .unwrap_or_else(|| INVALID_AUTHORITY.to_owned())
}

pub struct ConnectionPermit {
    throttle: NetworkThrottle,
    host: String,
}

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        self.throttle.release(&self.host);
    }
}
