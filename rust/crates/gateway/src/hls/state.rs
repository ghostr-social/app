use crate::hls::asset_generation::{AssetFence, AssetRegistry};
use crate::hls::types::{random_id, random_secret, HlsSessionId};
use core::time::Duration;
use ghostr_delivery::segmented::PreparedHlsPlaybackAsset;
use reqwest::Url;
use std::collections::HashMap;
use tokio::time::Instant;

#[derive(Default)]
pub(crate) struct HlsSessionState {
    pub sessions: HashMap<HlsSessionId, HlsSession>,
}

impl HlsSessionState {
    pub fn prune(&mut self, now: Instant, ttl: Duration) {
        self.sessions
            .retain(|_, session| now.duration_since(session.last_used) < ttl);
    }

    pub fn unique_id(&self) -> HlsSessionId {
        loop {
            let candidate = random_id();
            if !self.sessions.contains_key(&candidate) {
                return candidate;
            }
        }
    }

    pub fn active_session(
        &mut self,
        id: &HlsSessionId,
        now: Instant,
        ttl: Duration,
    ) -> Option<&mut HlsSession> {
        self.prune(now, ttl);
        let session = self.sessions.get_mut(id)?;
        session.last_used = now;
        Some(session)
    }
}

pub(crate) struct HlsSession {
    pub sources: Vec<Url>,
    pub last_used: Instant,
    pub secret: [u8; 32],
    assets: AssetRegistry,
    prepared: Option<PreparedHlsPlaybackAsset>,
}

impl HlsSession {
    pub fn new(sources: Vec<Url>, now: Instant) -> Self {
        Self {
            sources,
            last_used: now,
            secret: random_secret(),
            assets: AssetRegistry::new(),
            prepared: None,
        }
    }

    pub fn prepared(sources: Vec<Url>, asset: PreparedHlsPlaybackAsset, now: Instant) -> Self {
        let mut session = Self::new(sources, now);
        session.prepared = Some(asset);
        session
    }

    pub fn prepared_asset(&self) -> Option<PreparedHlsPlaybackAsset> {
        self.prepared.clone()
    }

    pub(in crate::hls) fn asset_fence(
        &mut self,
        url: &reqwest::Url,
        maximum: usize,
    ) -> anyhow::Result<AssetFence> {
        self.assets.fence(url, maximum)
    }

    pub(in crate::hls) fn owns(&self, fence: &AssetFence) -> bool {
        self.assets.owns(fence)
    }
}
