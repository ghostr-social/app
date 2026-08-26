use super::*;

impl SdkRelayIo {
    pub(crate) fn with_readiness_timeout(client: Arc<Client>, timeout: Duration) -> Self {
        Self::with_components(client, timeout)
    }
}
