use super::GatewayRuntime;
use crate::hls::playback::{HlsPlaybackRequest, NativeHlsPlaybackSession};

impl GatewayRuntime {
    /// Acquires a session pinned to one cache-certified HLS asset revision.
    ///
    /// # Errors
    ///
    /// Returns an error when authority is stale, sources differ, or capacity is exhausted.
    pub async fn acquire_hls(
        &self,
        request: HlsPlaybackRequest,
    ) -> anyhow::Result<NativeHlsPlaybackSession> {
        self.hls.acquire_prepared(&self.segmented, request).await
    }

    /// Compatibility path for callers without cache authority.
    ///
    /// # Errors
    ///
    /// Returns an error when sources are invalid or session capacity is exhausted.
    pub async fn acquire_unprepared_hls(
        &self,
        sources: Vec<String>,
    ) -> anyhow::Result<NativeHlsPlaybackSession> {
        self.hls.acquire(sources).await
    }

    pub async fn release_hls(&self, session_id: &str) -> bool {
        self.hls.release(session_id).await
    }
}
