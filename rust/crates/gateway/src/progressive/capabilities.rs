use anyhow::{ensure, Context as _, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use core::time::Duration;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_partial_store::partial_range_store::{ContentRevision, StoredMediaSnapshot};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio::time::Instant;

mod state;
use state::CapabilityState;

const TOKEN_BYTES: usize = 32;
const PRODUCTION_CAPACITY: usize = 256;
const PRODUCTION_IDLE_TTL: Duration = Duration::from_mins(30);

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
    /// Creates positive capacity and lifetime limits for progressive capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error when capacity is zero or the idle lifetime is empty.
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
    changed: Arc<Notify>,
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
            changed: Arc::new(Notify::new()),
        }
    }

    /// Issues or refreshes a capability for one stored representation.
    ///
    /// # Errors
    ///
    /// Returns an error when the snapshot has no representation binding.
    pub async fn issue(&self, snapshot: &StoredMediaSnapshot) -> Result<ProgressiveCapabilityId> {
        let authority = ProgressiveAssetAuthority::capture(snapshot)
            .context("progressive asset needs a representation binding")?;
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.prune(now, self.limits.idle_ttl);
        if let Some(id) = state.refresh(&authority, now) {
            return Ok(id);
        }
        state.make_room(self.limits.capacity);
        let id = state.insert(authority, now);
        drop(state);
        self.changed.notify_waiters();
        Ok(id)
    }

    /// Observes an issued capability without minting, refreshing, or evicting a live lease.
    pub async fn existing(
        &self,
        snapshot: &StoredMediaSnapshot,
    ) -> Option<ProgressiveCapabilityId> {
        let authority = ProgressiveAssetAuthority::capture(snapshot)?;
        let mut state = self.state.lock().await;
        state.prune(Instant::now(), self.limits.idle_ttl);
        state.existing(&authority)
    }

    /// Wakes observers when playback issues a new capability or clears all leases.
    pub fn notifier(&self) -> Arc<Notify> {
        Arc::clone(&self.changed)
    }

    pub(crate) async fn clear(&self) {
        *self.state.lock().await = CapabilityState::default();
        self.changed.notify_waiters();
    }

    pub async fn recognizes(&self, raw: &str, post: &str) -> bool {
        let Some(id) = ProgressiveCapabilityId::parse(raw) else {
            return false;
        };
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.prune(now, self.limits.idle_ttl);
        state.recognizes(&id, post)
    }

    pub async fn authorizes(&self, raw: &str, post: &str, snapshot: &StoredMediaSnapshot) -> bool {
        let Some(id) = ProgressiveCapabilityId::parse(raw) else {
            return false;
        };
        let Some(authority) = ProgressiveAssetAuthority::capture(snapshot) else {
            return false;
        };
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.prune(now, self.limits.idle_ttl);
        state.authorize(&id, post, &authority, now)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProgressiveAssetAuthority {
    binding: RepresentationBinding,
    revision: ContentRevision,
}

impl ProgressiveAssetAuthority {
    fn capture(snapshot: &StoredMediaSnapshot) -> Option<Self> {
        Some(Self {
            binding: snapshot.binding()?.clone(),
            revision: snapshot.revision(),
        })
    }

    fn post(&self) -> &str {
        self.binding.post().as_str()
    }
}
