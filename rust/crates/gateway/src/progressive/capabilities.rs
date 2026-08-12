use anyhow::{ensure, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

const TOKEN_BYTES: usize = 32;
const PRODUCTION_CAPACITY: usize = 256;
const PRODUCTION_IDLE_TTL: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProgressiveCapabilityId(String);

impl ProgressiveCapabilityId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn parse(raw: &str) -> Option<Self> {
        let decoded = URL_SAFE_NO_PAD.decode(raw).ok()?;
        (decoded.len() == TOKEN_BYTES).then(|| Self(raw.to_owned()))
    }
}

#[derive(Clone, Copy)]
pub struct ProgressiveCapabilityLimits {
    capacity: usize,
    idle_ttl: Duration,
}

impl ProgressiveCapabilityLimits {
    pub fn new(capacity: usize, idle_ttl: Duration) -> Result<Self> {
        ensure!(
            capacity > 0,
            "progressive capability capacity must be positive"
        );
        ensure!(
            !idle_ttl.is_zero(),
            "progressive capability TTL must be positive"
        );
        Ok(Self { capacity, idle_ttl })
    }
}

#[derive(Clone)]
pub struct ProgressiveCapabilities {
    limits: ProgressiveCapabilityLimits,
    state: Arc<Mutex<CapabilityState>>,
}

impl ProgressiveCapabilities {
    pub fn production() -> Self {
        let limits = ProgressiveCapabilityLimits::new(PRODUCTION_CAPACITY, PRODUCTION_IDLE_TTL)
            .expect("static progressive capability limits");
        Self::new(limits)
    }

    pub fn new(limits: ProgressiveCapabilityLimits) -> Self {
        Self {
            limits,
            state: Arc::new(Mutex::new(CapabilityState::default())),
        }
    }

    pub async fn issue(&self, post: &str) -> ProgressiveCapabilityId {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.prune(now, self.limits.idle_ttl);
        if let Some(id) = state.refresh_post(post, now) {
            return id;
        }
        state.make_room(self.limits.capacity);
        state.insert(post, now)
    }

    pub async fn authorizes(&self, raw: &str, post: &str) -> bool {
        let Some(id) = ProgressiveCapabilityId::parse(raw) else {
            return false;
        };
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.prune(now, self.limits.idle_ttl);
        state.authorize(&id, post, now)
    }
}

#[derive(Default)]
struct CapabilityState {
    entries: HashMap<ProgressiveCapabilityId, CapabilityLease>,
}

impl CapabilityState {
    fn prune(&mut self, now: Instant, ttl: Duration) {
        self.entries
            .retain(|_, lease| now.duration_since(lease.last_used) < ttl);
    }

    fn refresh_post(&mut self, post: &str, now: Instant) -> Option<ProgressiveCapabilityId> {
        let id = self
            .entries
            .iter()
            .find_map(|(id, lease)| (lease.post == post).then(|| id.clone()))?;
        self.entries.get_mut(&id)?.last_used = now;
        Some(id)
    }

    fn make_room(&mut self, capacity: usize) {
        if self.entries.len() < capacity {
            return;
        }
        let oldest = self
            .entries
            .iter()
            .min_by_key(|(_, lease)| lease.last_used)
            .map(|(id, _)| id.clone());
        if let Some(id) = oldest {
            self.entries.remove(&id);
        }
    }

    fn insert(&mut self, post: &str, now: Instant) -> ProgressiveCapabilityId {
        let id = self.unique_id();
        self.entries.insert(
            id.clone(),
            CapabilityLease {
                post: post.to_owned(),
                last_used: now,
            },
        );
        id
    }

    fn unique_id(&self) -> ProgressiveCapabilityId {
        loop {
            let candidate = random_id();
            if !self.entries.contains_key(&candidate) {
                return candidate;
            }
        }
    }

    fn authorize(&mut self, id: &ProgressiveCapabilityId, post: &str, now: Instant) -> bool {
        let Some(lease) = self.entries.get_mut(id) else {
            return false;
        };
        if lease.post != post {
            return false;
        }
        lease.last_used = now;
        true
    }
}

struct CapabilityLease {
    post: String,
    last_used: Instant,
}

fn random_id() -> ProgressiveCapabilityId {
    let mut bytes = [0_u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    ProgressiveCapabilityId(URL_SAFE_NO_PAD.encode(bytes))
}
