use crate::hls_sessions::{HlsSessionId, HlsSessions};
use anyhow::Result;
use std::net::SocketAddr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeHlsPlaybackSession {
    pub id: HlsSessionId,
    pub playback_url: String,
}

#[derive(Clone)]
pub struct HlsPlaybackGateway {
    endpoint: SocketAddr,
    sessions: HlsSessions,
}

impl HlsPlaybackGateway {
    pub fn new(endpoint: SocketAddr, sessions: HlsSessions) -> Self {
        Self { endpoint, sessions }
    }

    pub async fn acquire(&self, sources: Vec<String>) -> Result<NativeHlsPlaybackSession> {
        let id = self.sessions.acquire(sources).await?;
        let playback_url = format!("http://{}/hls/{}/index.m3u8", self.endpoint, id.as_str());
        Ok(NativeHlsPlaybackSession { id, playback_url })
    }

    pub async fn release(&self, raw_session_id: &str) -> bool {
        let Some(id) = HlsSessionId::parse(raw_session_id) else {
            return false;
        };
        self.sessions.release(&id).await
    }
}
