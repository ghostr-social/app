use crate::delivery_events::{
    DeliveryFocus, DeliveryPlayback, PlaybackPresentation, TransportRescueFeedback,
};
use crate::evaluation::{
    EvaluationLedger, PlaybackMetricEvent, PresentationMetricEvent, SemanticMetricEvent,
    SemanticMetricRollup,
};
use crate::manager::transfers::{InternalEvent, MaintenanceEvent};
use crate::qoe::{load_playback_learning, save_playback_learning, QoeTracker, WatchLearner};
use core::time::Duration;
use log::warn;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct QoeKeeper {
    tracker: QoeTracker,
    watch: WatchLearner,
    path: PathBuf,
    debounce: Duration,
    dirty: bool,
    save_pending: bool,
    evaluation: EvaluationLedger,
}

impl QoeKeeper {
    pub async fn load(path: PathBuf, debounce: Duration, evaluation: EvaluationLedger) -> Self {
        let learned = load_playback_learning(&path).await;
        Self {
            tracker: QoeTracker::from_stats(learned.qoe),
            watch: WatchLearner::from_model(learned.watch),
            path,
            debounce,
            dirty: false,
            save_pending: false,
            evaluation,
        }
    }

    pub fn note_focus(&mut self, focus: &DeliveryFocus, now_ms: u64) {
        self.watch.focus(focus, now_ms);
        let current = focus
            .items
            .get(focus.current_index.min(focus.items.len().saturating_sub(1)))
            .map(|item| item.post.clone());
        self.tracker
            .focus(current, focus.transition, focus.rescue.as_ref(), now_ms);
        self.evaluation.focus(
            focus
                .items
                .get(focus.current_index.min(focus.items.len().saturating_sub(1)))
                .map(|item| item.post.clone()),
            now_ms,
        );
        if let Some(rescue) = focus.rescue.as_ref() {
            self.evaluation.semantic(SemanticMetricEvent {
                rank_displacement: rescue.rank_displacement,
                semantic_regret_micros: u64::from(rescue.rank_displacement) * 1_000_000,
                transport_substitution: true,
            });
        }
        self.dirty = true;
    }

    pub fn note_rescue_feedback(&mut self, feedback: TransportRescueFeedback) {
        self.tracker.note_rescue_feedback(feedback);
        let rank_displacement = feedback.rank_displacement_total();
        self.evaluation.semantic_rollup(SemanticMetricRollup {
            rank_displacement,
            semantic_regret_micros: rank_displacement.saturating_mul(1_000_000),
            transport_substitutions: feedback.substitutions(),
        });
        self.dirty = true;
    }

    pub fn note_playback(&mut self, playback: &DeliveryPlayback, bitrate_bps: u64, now_ms: u64) {
        self.watch.playback(playback, now_ms);
        let buffer_ms = playback
            .observation
            .buffer_ahead()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        self.tracker.observe(
            playback.session.post(),
            playback.observation.phase(),
            buffer_ms,
            now_ms,
        );
        self.evaluation.playback(&PlaybackMetricEvent {
            post: playback.session.post().clone(),
            phase: playback.observation.phase(),
            bitrate_bps,
            observed_at_ms: now_ms,
        });
        self.dirty = true;
    }

    pub fn note_presentation(&mut self, event: &PlaybackPresentation, bitrate: u64, origin: &str) {
        self.tracker
            .present(event.session().post(), event.observed_at_ms());
        self.evaluation.present(&PresentationMetricEvent {
            post: event.session().post().clone(),
            bitrate_bps: bitrate,
            origin: origin.to_owned(),
            observed_at_ms: event.observed_at_ms(),
        });
        self.dirty = true;
    }

    pub fn startup_eta_ms(&self) -> u64 {
        self.tracker.stats().startup_eta_ms()
    }

    pub fn watch_model(&self) -> &ghostr_engine::watch_model::WatchModel {
        self.watch.model()
    }

    pub fn schedule_save(&mut self, events: &UnboundedSender<InternalEvent>) {
        if !self.dirty || self.save_pending {
            return;
        }
        self.save_pending = true;
        let events = events.clone();
        let debounce = self.debounce;
        tokio::spawn(async move {
            tokio::time::sleep(debounce).await;
            let _ = events.send(InternalEvent::Maintenance(MaintenanceEvent::SaveQoe));
        });
    }

    pub async fn save_now(&mut self) {
        self.save_pending = false;
        if !self.dirty {
            return;
        }
        match save_playback_learning(&self.path, self.tracker.stats(), self.watch.model()).await {
            Ok(()) => self.dirty = false,
            Err(error) => warn!("QoE aggregate snapshot failed: {error}"),
        }
    }
}
