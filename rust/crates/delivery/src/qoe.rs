//! Privacy-minimized playback learning: only bounded aggregate counters are
//! persisted. Media URLs, post ids, and raw event traces never enter the file.

mod persistence;

use crate::delivery_events::{FocusTransition, TransportRescue, TransportRescueReason};
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;
pub use persistence::{load_qoe_stats, save_qoe_stats};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct QoeStats {
    pub playback_sessions: u64,
    pub user_navigations: u64,
    pub first_frames: u64,
    pub startup_failures: u64,
    pub startup_total_ms: u64,
    pub startup_max_ms: u64,
    pub buffer_samples: u64,
    pub buffer_ahead_total_ms: u64,
    pub minimum_buffer_ahead_ms: Option<u64>,
    pub stall_events: u64,
    pub stall_total_ms: u64,
    pub decode_failures: u64,
    pub abandonments: u64,
    pub completions: u64,
    pub transport_substitutions: u64,
    pub rank_displacement_total: u64,
    pub rescue_wait_total_ms: u64,
    pub eta_unavailable_rescues: u64,
    pub eta_too_long_rescues: u64,
    pub delivery_failed_rescues: u64,
    pub grace_expired_rescues: u64,
}

impl QoeStats {
    pub fn startup_eta_ms(&self) -> u64 {
        if self.first_frames == 0 {
            return QoeTracker::DEFAULT_STARTUP_ETA_MS;
        }
        let mean = self.startup_total_ms / self.first_frames;
        mean.saturating_add(self.startup_max_ms.saturating_sub(mean) / 2)
            .clamp(50, 5_000)
    }
}

#[derive(Default)]
pub struct QoeTracker {
    stats: QoeStats,
    active: Option<ActivePlayback>,
}

struct ActivePlayback {
    post: PostId,
    focused_at_ms: u64,
    first_frame: bool,
    ended: bool,
    stall_started_ms: Option<u64>,
}

impl QoeTracker {
    pub const DEFAULT_STARTUP_ETA_MS: u64 = 750;

    pub fn from_stats(stats: QoeStats) -> Self {
        Self {
            stats,
            active: None,
        }
    }

    pub fn stats(&self) -> &QoeStats {
        &self.stats
    }

    pub fn focus(
        &mut self,
        post: Option<PostId>,
        transition: FocusTransition,
        rescue: Option<&TransportRescue>,
        now_ms: u64,
    ) {
        self.note_transition(transition, rescue);
        if self.active.as_ref().map(|active| &active.post) == post.as_ref() {
            return;
        }
        self.finish_active(now_ms);
        self.active = post.map(|post| ActivePlayback {
            post,
            focused_at_ms: now_ms,
            first_frame: false,
            ended: false,
            stall_started_ms: None,
        });
        self.stats.playback_sessions += u64::from(self.active.is_some());
    }

    pub fn observe(&mut self, post: &PostId, phase: PlaybackPhase, buffer_ms: u64, now_ms: u64) {
        let Some(active) = self.active.as_mut().filter(|active| &active.post == post) else {
            return;
        };
        record_buffer(&mut self.stats, phase, buffer_ms);
        match phase {
            PlaybackPhase::Playing => playing(&mut self.stats, active, now_ms),
            PlaybackPhase::NetworkStalled => stalled(&mut self.stats, active, now_ms),
            PlaybackPhase::Ended => ended(&mut self.stats, active, now_ms),
            PlaybackPhase::Failed => {
                self.stats.decode_failures += 1;
                self.finish_active(now_ms);
            }
            PlaybackPhase::Inactive => self.finish_active(now_ms),
            PlaybackPhase::Starting | PlaybackPhase::Paused => {
                close_stall(&mut self.stats, active, now_ms)
            }
        }
    }

    fn note_transition(&mut self, transition: FocusTransition, rescue: Option<&TransportRescue>) {
        if transition == FocusTransition::UserNavigation {
            self.stats.user_navigations += 1;
        }
        let Some(rescue) = rescue.filter(|_| transition == FocusTransition::TransportRescue) else {
            return;
        };
        self.stats.transport_substitutions += 1;
        self.stats.rank_displacement_total += u64::from(rescue.rank_displacement);
        self.stats.rescue_wait_total_ms += rescue.wait_ms;
        match rescue.reason {
            TransportRescueReason::EtaUnavailable => self.stats.eta_unavailable_rescues += 1,
            TransportRescueReason::EtaTooLong => self.stats.eta_too_long_rescues += 1,
            TransportRescueReason::DeliveryFailed => self.stats.delivery_failed_rescues += 1,
            TransportRescueReason::GraceExpired => self.stats.grace_expired_rescues += 1,
        }
    }

    fn finish_active(&mut self, now_ms: u64) {
        let Some(mut active) = self.active.take() else {
            return;
        };
        close_stall(&mut self.stats, &mut active, now_ms);
        self.stats.startup_failures += u64::from(!active.first_frame);
        self.stats.abandonments += u64::from(!active.ended);
    }
}

fn record_buffer(stats: &mut QoeStats, phase: PlaybackPhase, buffer_ms: u64) {
    if matches!(
        phase,
        PlaybackPhase::Ended | PlaybackPhase::Inactive | PlaybackPhase::Failed
    ) {
        return;
    }
    stats.buffer_samples += 1;
    stats.buffer_ahead_total_ms = stats.buffer_ahead_total_ms.saturating_add(buffer_ms);
    stats.minimum_buffer_ahead_ms = Some(
        stats
            .minimum_buffer_ahead_ms
            .map_or(buffer_ms, |old| old.min(buffer_ms)),
    );
}

fn playing(stats: &mut QoeStats, active: &mut ActivePlayback, now_ms: u64) {
    close_stall(stats, active, now_ms);
    if active.first_frame {
        return;
    }
    active.first_frame = true;
    let startup = now_ms.saturating_sub(active.focused_at_ms);
    stats.first_frames += 1;
    stats.startup_total_ms = stats.startup_total_ms.saturating_add(startup);
    stats.startup_max_ms = stats.startup_max_ms.max(startup);
}

fn stalled(stats: &mut QoeStats, active: &mut ActivePlayback, now_ms: u64) {
    if active.stall_started_ms.is_none() {
        active.stall_started_ms = Some(now_ms);
        stats.stall_events += 1;
    }
}

fn ended(stats: &mut QoeStats, active: &mut ActivePlayback, now_ms: u64) {
    close_stall(stats, active, now_ms);
    if !active.ended {
        active.ended = true;
        stats.completions += 1;
    }
}

fn close_stall(stats: &mut QoeStats, active: &mut ActivePlayback, now_ms: u64) {
    if let Some(started) = active.stall_started_ms.take() {
        stats.stall_total_ms = stats
            .stall_total_ms
            .saturating_add(now_ms.saturating_sub(started));
    }
}
