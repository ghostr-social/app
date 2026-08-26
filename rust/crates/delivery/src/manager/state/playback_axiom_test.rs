use super::*;

impl DeliveryState {
    pub(crate) fn apply_playback(&mut self, update: &DeliveryPlayback) -> PlaybackAdmission {
        self.apply_playback_at(update, 0)
    }
}
