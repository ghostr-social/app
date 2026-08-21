//! Always-on admission for outbound media requests.
//!
//! The low-level client remains a request-builder port; this executor is the
//! guarded surface production media consumers will migrate to. Admission and
//! sending are split so queue delay is never misreported as origin response
//! time.

mod gate;
mod request;
mod response;

use crate::outbound_media_client::MediaHttpRequests;
use anyhow::{ensure, Context, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use std::num::NonZeroUsize;
use std::sync::Arc;

use gate::MediaRequestGate;
pub use request::{AdmittedMediaRequest, MediaRequest};
pub use response::MediaResponse;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaRequestLimits {
    global: NonZeroUsize,
    per_authority: NonZeroUsize,
}

impl MediaRequestLimits {
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

    pub const fn global(self) -> usize {
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
    pub fn new(client: Arc<dyn MediaHttpRequests>, limits: MediaRequestLimits) -> Self {
        Self {
            client,
            gate: MediaRequestGate::new(limits),
        }
    }

    pub fn get(&self, raw_url: &str, priority: PreemptionAuthority) -> Result<MediaRequest> {
        let authority =
            RequestAuthority::from_url(raw_url).context("media request authority is invalid")?;
        let builder = self.client.get(raw_url)?;
        Ok(MediaRequest::new(
            builder,
            authority,
            priority,
            self.gate.clone(),
        ))
    }

    pub fn update_limits(&self, limits: MediaRequestLimits) {
        self.gate.update_limits(limits);
    }

    pub fn limits(&self) -> MediaRequestLimits {
        self.gate.limits()
    }
}
