use crate::delivery_events::DeliveryPlayback;
use crate::evaluation::IntegrityMetricEvent;
use crate::manager::time::unix_time_ms;
use crate::manager::DeliveryWorker;
use ghostr_engine::concurrency::NetworkSetback;
use ghostr_engine::playback::{PlaybackPhase, PlaybackStatus};

#[cfg(test)]
#[path = "playback/stall_episode_test.rs"]
mod stall_episode_test;
#[cfg(test)]
#[path = "playback/test_support.rs"]
mod test_support;

impl DeliveryWorker {
    pub(super) fn apply_playback(&mut self, playback: &DeliveryPlayback) {
        let stalled = starts_stall_episode(self.state.playback(), playback);
        let post = playback.session.post().clone();
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
            self.qoe.note_playback(playback, bitrate, observed_at_ms);
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

fn starts_stall_episode(previous: &PlaybackStatus, update: &DeliveryPlayback) -> bool {
    update.observation.phase() == PlaybackPhase::NetworkStalled
        && (previous.session() != Some(&update.session)
            || previous
                .observation()
                .is_none_or(|value| value.phase() != PlaybackPhase::NetworkStalled))
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
