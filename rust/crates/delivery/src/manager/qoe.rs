use crate::delivery_events::{DeliveryFocus, DeliveryPlayback};
use crate::manager::transfers::{InternalEvent, MaintenanceEvent};
use crate::qoe::{load_qoe_stats, save_qoe_stats, QoeTracker};
use log::warn;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct QoeKeeper {
    tracker: QoeTracker,
    path: PathBuf,
    debounce: Duration,
    dirty: bool,
    save_pending: bool,
}

impl QoeKeeper {
    pub async fn load(path: PathBuf, debounce: Duration) -> Self {
        let stats = load_qoe_stats(&path).await;
        Self {
            tracker: QoeTracker::from_stats(stats),
            path,
            debounce,
            dirty: false,
            save_pending: false,
        }
    }

    pub fn note_focus(&mut self, focus: &DeliveryFocus, now_ms: u64) {
        let current = focus
            .items
            .get(focus.current_index.min(focus.items.len().saturating_sub(1)))
            .map(|item| item.post.clone());
        self.tracker
            .focus(current, focus.transition, focus.rescue.as_ref(), now_ms);
        self.dirty = true;
    }

    pub fn note_playback(&mut self, playback: &DeliveryPlayback, now_ms: u64) {
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
        self.dirty = true;
    }

    pub fn startup_eta_ms(&self) -> u64 {
        self.tracker.stats().startup_eta_ms()
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
        match save_qoe_stats(&self.path, self.tracker.stats()).await {
            Ok(()) => self.dirty = false,
            Err(error) => warn!("QoE aggregate snapshot failed: {error}"),
        }
    }
}
