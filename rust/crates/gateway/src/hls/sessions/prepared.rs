use super::{HlsSession, HlsSessionId, HlsSessions};
use crate::hls::playback::HlsPlaybackRequest;
use anyhow::{Context as _, Result};
use ghostr_delivery::segmented::{
    HlsPreparedAssetAuthority, PreparedHlsPlaybackAsset, SegmentedCache,
};
use reqwest::Url;
use tokio::time::Instant;

#[derive(Clone)]
pub(crate) enum HlsPlaybackBinding {
    Unprepared(Vec<Url>),
    Prepared(PreparedHlsPlaybackAsset),
}

impl HlsSessions {
    pub async fn acquire_prepared(
        &self,
        cache: &SegmentedCache,
        request: HlsPlaybackRequest,
    ) -> Result<HlsSessionId> {
        let asset = cache
            .capture_prepared_asset(&request.authority, &request.raw_sources)
            .context("prepared HLS authority is unavailable")?;
        self.admit(HlsSession::prepared(request.sources, asset, Instant::now()))
            .await
    }

    pub async fn authority(&self, id: &HlsSessionId) -> Option<HlsPreparedAssetAuthority> {
        self.active_binding(id)
            .await?
            .prepared
            .map(|asset| asset.authority().clone())
    }

    pub(crate) async fn playback_binding(&self, id: &HlsSessionId) -> Option<HlsPlaybackBinding> {
        let session = self.active_binding(id).await?;
        Some(match session.prepared {
            Some(asset) => HlsPlaybackBinding::Prepared(asset),
            None => HlsPlaybackBinding::Unprepared(session.sources),
        })
    }

    async fn active_binding(&self, id: &HlsSessionId) -> Option<HlsSessionBinding> {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        let session = state.active_session(id, now, self.limits.idle_ttl)?;
        Some(HlsSessionBinding {
            sources: session.sources.clone(),
            prepared: session.prepared_asset(),
        })
    }
}

struct HlsSessionBinding {
    sources: Vec<Url>,
    prepared: Option<PreparedHlsPlaybackAsset>,
}
