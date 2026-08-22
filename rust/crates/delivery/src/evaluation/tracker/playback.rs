use super::{ratio, ActivePlayback, EvaluationTracker};
use crate::evaluation::events::{PlaybackMetricEvent, PresentationMetricEvent};
use crate::evaluation::types::EvaluationSnapshot;
use ghostr_engine::host_stats::host_of;
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

impl EvaluationTracker {
    pub fn focus(&mut self, post: PostId, observed_at_ms: u64) {
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.post == post)
        {
            return;
        }
        self.finish_active(observed_at_ms);
        self.active = Some(ActivePlayback {
            post,
            focused_at_ms: observed_at_ms,
            last_at_ms: observed_at_ms,
            phase: PlaybackPhase::Starting,
            bitrate_bps: 0,
            presented: false,
            startup_failure_counted: false,
        });
        self.metrics.user_visible.startup_sessions += 1;
        self.metrics.semantics.focus_sessions += 1;
    }

    pub fn playback(&mut self, event: PlaybackMetricEvent) {
        let Some(active) = self.active.as_ref().filter(|item| item.post == event.post) else {
            return;
        };
        let old_phase = active.phase;
        let old_bitrate = active.bitrate_bps;
        let presented = active.presented;
        self.account_until(event.observed_at_ms);
        if presented
            && event.phase == PlaybackPhase::NetworkStalled
            && old_phase != PlaybackPhase::NetworkStalled
        {
            self.metrics.user_visible.stall_events += 1;
        }
        if presented && old_bitrate > 0 && old_bitrate != event.bitrate_bps {
            self.metrics.user_visible.quality_discontinuities += 1;
        }
        if let Some(active) = self.active.as_mut() {
            active.phase = event.phase;
            active.bitrate_bps = event.bitrate_bps;
        }
        if event.phase == PlaybackPhase::Failed {
            self.startup_failure();
        }
    }

    pub fn present(&mut self, event: PresentationMetricEvent) {
        let Some(active) = self.active.as_ref().filter(|item| item.post == event.post) else {
            return;
        };
        if active.presented || active.startup_failure_counted {
            return;
        }
        let latency = event.observed_at_ms.saturating_sub(active.focused_at_ms);
        self.account_until(event.observed_at_ms);
        self.first_frame_latency.push(latency);
        self.first_frame_quality_sum = self
            .first_frame_quality_sum
            .saturating_add(u128::from(event.bitrate_bps));
        self.first_frame_quality_count += 1;
        self.metrics.efficiency.playable_videos += 1;
        self.unused_full_downloads.remove(&event.post);
        self.note_exposure(&event.origin);
        if let Some(active) = self.active.as_mut() {
            active.presented = true;
            active.bitrate_bps = event.bitrate_bps;
        }
    }

    pub fn finish(&mut self, observed_at_ms: u64) {
        self.finish_active(observed_at_ms);
    }

    fn account_until(&mut self, observed_at_ms: u64) {
        let Some((elapsed, phase, bitrate, presented)) = self.active.as_mut().map(|active| {
            let elapsed = observed_at_ms.saturating_sub(active.last_at_ms);
            active.last_at_ms = active.last_at_ms.max(observed_at_ms);
            (elapsed, active.phase, active.bitrate_bps, active.presented)
        }) else {
            return;
        };
        if !presented {
            return;
        }
        match phase {
            PlaybackPhase::Playing => self.note_played(elapsed, bitrate),
            PlaybackPhase::NetworkStalled => {
                self.metrics.user_visible.stall_ms =
                    self.metrics.user_visible.stall_ms.saturating_add(elapsed);
            }
            _ => {}
        }
    }

    fn note_played(&mut self, elapsed_ms: u64, bitrate_bps: u64) {
        self.played_ms = self.played_ms.saturating_add(elapsed_ms);
        self.watch_quality_sum = self
            .watch_quality_sum
            .saturating_add(u128::from(elapsed_ms) * u128::from(bitrate_bps));
        let bytes = u128::from(elapsed_ms)
            .saturating_mul(u128::from(bitrate_bps))
            .saturating_div(8_000)
            .min(u128::from(u64::MAX)) as u64;
        self.metrics.efficiency.useful_watched_bytes = self
            .metrics
            .efficiency
            .useful_watched_bytes
            .saturating_add(bytes);
    }

    fn finish_active(&mut self, observed_at_ms: u64) {
        self.account_until(observed_at_ms);
        self.startup_failure();
        self.active = None;
    }

    fn startup_failure(&mut self) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        if active.presented || active.startup_failure_counted {
            return;
        }
        active.startup_failure_counted = true;
        self.metrics.user_visible.startup_failures += 1;
    }

    fn note_exposure(&mut self, origin: &str) {
        const ORIGIN_CAPACITY: usize = 64;
        let origin = host_of(origin).unwrap_or_else(|| origin.to_ascii_lowercase());
        let origin = self.privacy.origin(&origin);
        let exposures = &mut self.metrics.semantics.exposure_by_origin;
        if exposures.contains_key(&origin) || exposures.len() < ORIGIN_CAPACITY {
            *exposures.entry(origin).or_default() += 1;
        }
    }
}

pub(super) fn populate(tracker: &EvaluationTracker, output: &mut EvaluationSnapshot) {
    let visible = &mut output.user_visible;
    visible.startup_failure_rate_bps = ratio(visible.startup_failures, visible.startup_sessions);
    visible.stall_ratio_bps = ratio(visible.stall_ms, tracker.played_ms + visible.stall_ms);
    visible.first_frame_quality_bps = mean(
        tracker.first_frame_quality_sum,
        tracker.first_frame_quality_count,
    );
    visible.watch_weighted_quality_bps = mean(tracker.watch_quality_sum, tracker.played_ms);
    output.efficiency.requests_per_playable_milli = rate(
        output.efficiency.request_count,
        output.efficiency.playable_videos,
    );
    output.semantics.transport_substitution_rate_bps = ratio(
        output.semantics.transport_substitutions,
        output.semantics.focus_sessions,
    );
}

fn mean(sum: u128, count: u64) -> u64 {
    sum.checked_div(u128::from(count))
        .unwrap_or_default()
        .min(u128::from(u64::MAX)) as u64
}

fn rate(value: u64, count: u64) -> u64 {
    value
        .saturating_mul(1_000)
        .checked_div(count)
        .unwrap_or_default()
}
