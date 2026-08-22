use crate::delivery_events::DeliveryPlayback;
use crate::evaluation::IntegrityMetricEvent;
use crate::manager::time::unix_time_ms;
use crate::manager::DeliveryWorker;
use ghostr_engine::concurrency::NetworkSetback;
use ghostr_engine::playback::PlaybackPhase;

impl DeliveryWorker {
    pub(crate) fn apply_playback(&mut self, playback: DeliveryPlayback) {
        let stalled = playback.observation.phase() == PlaybackPhase::NetworkStalled;
        let post = playback.session.post().clone();
        let evidence = playback.clone();
        let observed_at_ms = unix_time_ms();
        let false_streamability = stalled_or_failed_streamability(
            &self.state,
            &post,
            playback.observation.phase(),
            observed_at_ms,
        );
        let admission = self.state.apply_playback_at(playback, observed_at_ms);
        self.commands.record_playback_admission(admission, &post);
        if admission.is_accepted() {
            let bitrate = self
                .state
                .catalog()
                .estimated_bitrate(&post, self.state.params());
            self.apply_pending_presentation();
            self.qoe.note_playback(&evidence, bitrate, observed_at_ms);
            if false_streamability {
                self.commands
                    .evaluation()
                    .integrity(IntegrityMetricEvent::FalseStreamability);
            }
        }
        if admission.is_accepted() && stalled {
            self.note_network_setback(NetworkSetback::Stall);
        }
    }
}

fn stalled_or_failed_streamability(
    state: &crate::manager::state::DeliveryState,
    post: &ghostr_engine::PostId,
    phase: PlaybackPhase,
    now_ms: u64,
) -> bool {
    if phase != PlaybackPhase::Failed {
        return false;
    }
    state.catalog().lookup(post).is_some_and(|entry| {
        entry.meta.urls.first().is_some_and(|source| {
            entry
                .evidence_assessment_for(source, now_ms)
                .confidence
                .readiness
                .basis_points()
                > 0
        })
    })
}
