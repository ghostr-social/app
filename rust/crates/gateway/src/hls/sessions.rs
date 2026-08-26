use crate::hls::asset_generation::AssetFence;
use crate::hls::capability::{issue, open};
use crate::hls::state::{HlsSession, HlsSessionState};
use crate::hls::types::validated_sources;
use anyhow::{bail, Context as _, Result};
use core::time::Duration;
use ghostr_hls_manifest::hls_manifest::{rewrite_hls_manifest, HlsResource, HlsResourceKind};
use reqwest::Url;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout_at, Instant};

pub use crate::hls::types::{HlsResourceId, HlsSessionId, HlsSessionLimits};

#[derive(Clone)]
pub struct HlsSessions {
    limits: HlsSessionLimits,
    state: Arc<Mutex<HlsSessionState>>,
}

impl HlsSessions {
    pub fn production() -> Self {
        let limits = HlsSessionLimits::new(32, Duration::from_mins(30), 1_024)
            .expect("static HLS session limits");
        Self::new(limits)
    }

    pub fn new(limits: HlsSessionLimits) -> Self {
        Self {
            limits,
            state: Arc::new(Mutex::new(HlsSessionState::default())),
        }
    }

    /// Opens a bounded session for validated HLS source URLs.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid sources or exhausted session capacity.
    pub async fn acquire(&self, sources: Vec<String>) -> Result<HlsSessionId> {
        let sources = validated_sources(sources)?;
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.prune(now, self.limits.idle_ttl);
        if state.sessions.len() >= self.limits.max_sessions {
            bail!("secure HLS session capacity is exhausted");
        }
        let id = state.unique_id();
        state
            .sessions
            .insert(id.clone(), HlsSession::new(sources, now));
        Ok(id)
    }

    pub async fn sources(&self, id: &HlsSessionId) -> Option<Vec<Url>> {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        let session = state.active_session(id, now, self.limits.idle_ttl)?;
        Some(session.sources.clone())
    }

    pub async fn resource(
        &self,
        session: &HlsSessionId,
        resource: HlsResourceId,
    ) -> Option<HlsResource> {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        let session = state.active_session(session, now, self.limits.idle_ttl)?;
        let secret = session.secret;
        drop(state);
        open(&secret, resource)
    }

    pub(in crate::hls) async fn resource_at(
        &self,
        session: &HlsSessionId,
        resource: HlsResourceId,
        deadline: Instant,
    ) -> Result<Option<HlsResource>> {
        let mut state = timeout_at(deadline, self.state.lock())
            .await
            .context("HLS session lookup timed out")?;
        let Some(session) = state.active_session(session, Instant::now(), self.limits.idle_ttl)
        else {
            return Ok(None);
        };
        Ok(open(&session.secret, resource))
    }

    pub(in crate::hls) async fn asset_fence_at(
        &self,
        session: &HlsSessionId,
        url: &Url,
        deadline: Instant,
    ) -> Result<Option<AssetFence>> {
        let mut state = timeout_at(deadline, self.state.lock())
            .await
            .context("HLS asset fence lookup timed out")?;
        let Some(session) = state.active_session(session, Instant::now(), self.limits.idle_ttl)
        else {
            return Ok(None);
        };
        session
            .asset_fence(url, self.limits.max_ranged_assets)
            .map(Some)
    }

    pub(in crate::hls) async fn owns_asset_at(
        &self,
        session: &HlsSessionId,
        fence: &AssetFence,
        deadline: Instant,
    ) -> Result<bool> {
        let mut state = timeout_at(deadline, self.state.lock())
            .await
            .context("HLS session ownership check timed out")?;
        Ok(state
            .active_session(session, Instant::now(), self.limits.idle_ttl)
            .is_some_and(|active| active.owns(fence)))
    }

    /// Rewrites a manifest with authenticated, session-scoped resource paths.
    ///
    /// # Errors
    ///
    /// Returns an error when the session is unavailable or manifest rewriting fails.
    pub async fn rewrite_manifest(
        &self,
        id: &HlsSessionId,
        body: &[u8],
        base_url: &Url,
    ) -> Result<String> {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        let session = state
            .active_session(id, now, self.limits.idle_ttl)
            .ok_or_else(|| anyhow::anyhow!("secure HLS session is unavailable"))?;
        let secret = session.secret;
        drop(state);
        rewrite_hls_manifest(body, base_url, |resource| {
            let kind = resource.kind;
            let resource = issue(&secret, &resource)?;
            Ok(resource_path(id, &resource, kind))
        })
    }

    pub async fn release(&self, id: &HlsSessionId) -> bool {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.prune(now, self.limits.idle_ttl);
        state.sessions.remove(id).is_some()
    }

    #[cfg(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    ))]
    pub(crate) async fn clear(&self) {
        self.state.lock().await.sessions.clear();
    }
}

fn resource_path(
    session: &HlsSessionId,
    resource: &HlsResourceId,
    kind: HlsResourceKind,
) -> String {
    match kind {
        HlsResourceKind::Manifest => {
            format!("/hls/{}/manifests/{resource}/index.m3u8", session.as_str())
        }
        HlsResourceKind::Asset => {
            format!("/hls/{}/assets/{resource}", session.as_str())
        }
    }
}
