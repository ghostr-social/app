//! Always-on admission for outbound media requests.
//!
//! The low-level client is a one-hop request-builder port; this executor is the
//! guarded production surface. Admission and sending are split so queue delay
//! is never misreported as origin response time, and redirects reacquire the
//! exact authority before the next network hop.

mod body_limit;
mod gate;
mod redirect;
mod request;
mod response;

use crate::internet_allowance::InternetAllowance;
use crate::outbound_media_client::MediaHttpRequests;
use anyhow::{ensure, Context as _, Result};
use core::num::NonZeroUsize;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use std::sync::Arc;

use gate::MediaRequestGate;
use request::RequestRoute;
pub use request::{AdmittedMediaRequest, MediaRequest, MediaRequestAdmissionTimeout};
pub use response::MediaResponse;

pub trait MediaResourceObserver: Send + Sync {
    fn record_request(&self);
    fn record_response_bytes(&self, bytes: u64);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaRequestLimits {
    global: NonZeroUsize,
    per_authority: NonZeroUsize,
}

impl MediaRequestLimits {
    /// # Errors
    ///
    /// Returns an error when a limit is zero or the per-authority limit exceeds the global limit.
    pub fn try_new(global: usize, per_authority: usize) -> Result<Self> {
        let global = NonZeroUsize::new(global).context("global request limit is zero")?;
        let per_authority =
            NonZeroUsize::new(per_authority).context("authority request limit is zero")?;
        ensure!(
            per_authority <= global,
            "authority request limit exceeds global limit"
        );
        Ok(Self {
            global,
            per_authority,
        })
    }

    const fn global(self) -> usize {
        self.global.get()
    }

    pub const fn per_authority(self) -> usize {
        self.per_authority.get()
    }
}

#[derive(Clone)]
pub struct MediaRequestExecutor {
    client: Arc<dyn MediaHttpRequests>,
    gate: MediaRequestGate,
}

impl MediaRequestExecutor {
    pub fn with_allowance(
        client: Arc<dyn MediaHttpRequests>,
        limits: MediaRequestLimits,
        allowance: InternetAllowance,
    ) -> Self {
        Self {
            client,
            gate: MediaRequestGate::new(limits, allowance),
        }
    }

    /// # Errors
    ///
    /// Returns an error when `raw_url` is unsafe or a request cannot be built.
    pub fn get(&self, raw_url: &str, priority: PreemptionAuthority) -> Result<MediaRequest> {
        let route = RequestRoute::new(
            Arc::clone(&self.client),
            self.gate.clone(),
            raw_url,
            priority,
        )?;
        let builder = self.client.get(raw_url)?;
        Ok(MediaRequest::new(builder, route))
    }

    pub fn update_limits(&self, limits: MediaRequestLimits) {
        self.gate.update_limits(limits);
    }

    pub fn install_resource_observer(&self, observer: Arc<dyn MediaResourceObserver>) -> bool {
        self.gate.install_resource_observer(observer)
    }

    pub fn limits(&self) -> MediaRequestLimits {
        self.gate.limits()
    }

    pub fn active_for(&self, authority: &RequestAuthority) -> usize {
        self.gate.active_for(authority)
    }
}

#[cfg(any(test, feature = "test"))]
mod test_support;
