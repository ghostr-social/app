use crate::hls::sessions::{HlsSessionId, HlsSessions};
use crate::hls::types::validated_sources;
use anyhow::Result;
use core::net::SocketAddr;
use ghostr_delivery::segmented::HlsPreparedAssetAuthority;
use reqwest::Url;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeHlsPlaybackSession {
    pub id: HlsSessionId,
    pub playback_url: String,
    pub authority: Option<HlsPreparedAssetAuthority>,
}

pub struct HlsPlaybackRequest {
    pub(crate) authority: HlsPreparedAssetAuthority,
    pub(crate) raw_sources: Vec<String>,
    pub(crate) sources: Vec<Url>,
}

impl HlsPlaybackRequest {
    pub fn new(authority: HlsPreparedAssetAuthority, sources: Vec<String>) -> Result<Self> {
        let parsed = validated_sources(sources.clone())?;
        Ok(Self {
            authority,
            raw_sources: sources,
            sources: parsed,
        })
    }
}

#[derive(Clone)]
pub struct HlsPlaybackGateway {
    endpoint: SocketAddr,
    sessions: HlsSessions,
}

impl HlsPlaybackGateway {
    pub(crate) fn new(endpoint: SocketAddr, sessions: HlsSessions) -> Self {
        Self { endpoint, sessions }
    }

    pub(crate) async fn acquire(&self, sources: Vec<String>) -> Result<NativeHlsPlaybackSession> {
        let id = self.sessions.acquire(sources).await?;
        Ok(self.session(id, None))
    }

    pub(crate) async fn acquire_prepared(
        &self,
        cache: &ghostr_delivery::segmented::SegmentedCache,
        request: HlsPlaybackRequest,
    ) -> Result<NativeHlsPlaybackSession> {
        let authority = request.authority.clone();
        let id = self.sessions.acquire_prepared(cache, request).await?;
        Ok(self.session(id, Some(authority)))
    }

    pub(crate) async fn release(&self, raw_session_id: &str) -> bool {
        let Some(id) = HlsSessionId::parse(raw_session_id) else {
            return false;
        };
        self.sessions.release(&id).await
    }

    fn session(
        &self,
        id: HlsSessionId,
        authority: Option<HlsPreparedAssetAuthority>,
    ) -> NativeHlsPlaybackSession {
        let playback_url = format!("http://{}/hls/{}/index.m3u8", self.endpoint, id.as_str());
        NativeHlsPlaybackSession {
            id,
            playback_url,
            authority,
        }
    }
}
