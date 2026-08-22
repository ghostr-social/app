use crate::hls::asset_response::AssetResponseEnvelope;
use anyhow::{bail, Context, Result};
use ghostr_delivery::segmented::CachedHlsGeneration;
use ghostr_net::media_request_executor::MediaResponse;
use reqwest::Url;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};
use tokio::time::{timeout_at, Instant};

const KEY_DOMAIN: &[u8] = b"ghostr:hls-asset-key:v1";

mod origin;
pub(in crate::hls) use origin::OriginGeneration;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct AssetKey([u8; 32]);

#[derive(Debug)]
enum AssetBinding {
    Vacant,
    Origin(OriginGeneration),
    Cache(CachedHlsGeneration),
    Retired,
}

enum BindingPlan {
    First,
    Origin(OriginGeneration),
    Cache(CachedHlsGeneration),
}

pub(super) struct AssetRegistry {
    owner: Arc<()>,
    entries: HashMap<AssetKey, Arc<Mutex<AssetBinding>>>,
}

#[derive(Clone)]
pub(super) struct AssetFence {
    owner: Arc<()>,
    state: Arc<Mutex<AssetBinding>>,
}

pub(super) enum AssetPlan {
    First(FirstAdmission),
    Origin(OriginGeneration),
    Cache(CachedHlsGeneration),
}

pub(super) struct FirstAdmission(OwnedMutexGuard<AssetBinding>);

pub(in crate::hls) struct OriginConfirmation<'a> {
    pub expected: &'a OriginGeneration,
    pub envelope: AssetResponseEnvelope,
    pub response: &'a MediaResponse,
    pub deadline: Instant,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            owner: Arc::new(()),
            entries: HashMap::new(),
        }
    }

    pub fn fence(&mut self, url: &Url, maximum: usize) -> Result<AssetFence> {
        let key = AssetKey(fingerprint(KEY_DOMAIN, url));
        if !self.entries.contains_key(&key) && self.entries.len() >= maximum {
            bail!("secure HLS asset generation capacity is exhausted");
        }
        let state = self
            .entries
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(AssetBinding::Vacant)))
            .clone();
        Ok(AssetFence {
            owner: self.owner.clone(),
            state,
        })
    }

    pub fn owns(&self, fence: &AssetFence) -> bool {
        Arc::ptr_eq(&self.owner, &fence.owner)
    }
}

impl AssetFence {
    pub async fn plan(
        &self,
        cached: Option<CachedHlsGeneration>,
        deadline: Instant,
    ) -> Result<AssetPlan> {
        let mut binding = self.lock(deadline).await?;
        match binding.next(cached)? {
            BindingPlan::First => Ok(AssetPlan::First(FirstAdmission(binding))),
            BindingPlan::Origin(generation) => Ok(AssetPlan::Origin(generation)),
            BindingPlan::Cache(generation) => Ok(AssetPlan::Cache(generation)),
        }
    }

    pub async fn confirm_origin(&self, confirmation: OriginConfirmation<'_>) -> Result<()> {
        let mut binding = self.lock(confirmation.deadline).await?;
        if !matches!(&*binding, AssetBinding::Origin(found) if found == confirmation.expected) {
            bail!("HLS asset generation changed concurrently");
        }
        if !confirmation
            .expected
            .matches(confirmation.envelope, confirmation.response)
        {
            *binding = AssetBinding::Retired;
            bail!("HLS asset response changed generation");
        }
        Ok(())
    }

    pub async fn retire_origin(
        &self,
        expected: &OriginGeneration,
        deadline: Instant,
    ) -> Result<()> {
        let mut binding = self.lock(deadline).await?;
        if matches!(&*binding, AssetBinding::Origin(found) if found == expected) {
            *binding = AssetBinding::Retired;
        }
        Ok(())
    }

    async fn lock(&self, deadline: Instant) -> Result<OwnedMutexGuard<AssetBinding>> {
        timeout_at(deadline, self.state.clone().lock_owned())
            .await
            .context("HLS asset generation wait timed out")
    }
}

impl AssetBinding {
    fn next(&mut self, cached: Option<CachedHlsGeneration>) -> Result<BindingPlan> {
        match self {
            Self::Vacant => self.bind_first(cached),
            Self::Origin(generation) => Ok(BindingPlan::Origin(generation.clone())),
            Self::Cache(_) => self.reuse_cache(cached),
            Self::Retired => bail!("HLS asset generation is retired"),
        }
    }

    fn bind_first(&mut self, cached: Option<CachedHlsGeneration>) -> Result<BindingPlan> {
        let Some(generation) = cached else {
            return Ok(BindingPlan::First);
        };
        *self = Self::Cache(generation);
        Ok(BindingPlan::Cache(generation))
    }

    fn reuse_cache(&mut self, cached: Option<CachedHlsGeneration>) -> Result<BindingPlan> {
        let Self::Cache(expected) = self else {
            unreachable!("cache reuse requires a cache binding");
        };
        let expected = *expected;
        if cached == Some(expected) {
            return Ok(BindingPlan::Cache(expected));
        }
        *self = Self::Retired;
        bail!("cached HLS asset generation changed")
    }
}

impl FirstAdmission {
    pub fn admit(mut self, envelope: AssetResponseEnvelope, response: &MediaResponse) {
        if let AssetResponseEnvelope::Partial { total, .. } = envelope {
            *self.0 = OriginGeneration::observed(response, total)
                .map_or(AssetBinding::Retired, AssetBinding::Origin);
        }
    }
}

fn fingerprint(domain: &[u8], url: &Url) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(url.as_str().as_bytes());
    digest.finalize().into()
}
