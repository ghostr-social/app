use crate::delivery_events::PlaybackPresentation;
use crate::manager::state::PresentationAdmission;
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(super) fn apply_presentation(&mut self, event: &PlaybackPresentation) {
        if self.state.apply_presentation(event) == PresentationAdmission::Accepted {
            let (bitrate, origin) = self.presentation_metrics(event.session().post());
            self.qoe.note_presentation(event, bitrate, &origin);
            self.qoe.schedule_save(&self.ctx.events);
        }
    }

    pub(super) fn apply_pending_presentation(&mut self) {
        if let Some(event) = self.state.take_pending_presentation() {
            let (bitrate, origin) = self.presentation_metrics(event.session().post());
            self.qoe.note_presentation(&event, bitrate, &origin);
        }
    }

    fn presentation_metrics(&self, post: &ghostr_engine::PostId) -> (u64, String) {
        let bitrate = self
            .state
            .catalog()
            .estimated_bitrate(post, self.state.params());
        let origin = self
            .state
            .catalog()
            .lookup(post)
            .and_then(|entry| entry.meta.urls.first())
            .cloned()
            .unwrap_or_else(|| "local".into());
        (bitrate, origin)
    }
}
