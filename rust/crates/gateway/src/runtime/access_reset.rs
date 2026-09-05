use super::GatewayRuntime;

impl GatewayRuntime {
    /// Revokes local playback sessions and releases private media buffers.
    ///
    /// # Errors
    /// Returns an error when the delivery worker cannot acknowledge cleanup.
    pub async fn reset_playback_access(&self) -> anyhow::Result<()> {
        self.hls.clear().await;
        self.progressive.capabilities.clear().await;
        self.delivery.reset_playback_access().await
    }
}
