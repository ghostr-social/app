use super::DeliveryState;
use ghostr_engine::catalog::PlaybackEvidence;
use ghostr_engine::PostId;

const PLAYBACK_CLIENT: &str = concat!("ghostr-native-player/", env!("CARGO_PKG_VERSION"));

impl DeliveryState {
    pub(super) fn learn_playback_readiness(
        &mut self,
        post: &PostId,
        first_frame: bool,
        observed_at_ms: u64,
    ) {
        let Some(binding) = self.catalog.binding(post) else {
            return;
        };
        self.catalog.learn_playback_for(
            &binding,
            PlaybackEvidence::new(PLAYBACK_CLIENT, first_frame, observed_at_ms),
        );
    }
}
