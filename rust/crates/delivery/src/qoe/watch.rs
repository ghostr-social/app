use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusTransition};
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::watch_model::{
    WatchCensor, WatchContext, WatchModel, WatchNavigation, WatchSample, WatchSampleKind,
};
use ghostr_engine::PostId;

mod focus;

use focus::{departure_kind, focused, navigation, same_post};

#[derive(Clone, Debug)]
struct ActiveWatch {
    post: PostId,
    context: WatchContext,
    watched_ms: u64,
    terminal: bool,
    generation: u64,
}

#[derive(Default)]
pub(crate) struct WatchLearner {
    model: WatchModel,
    active: Option<ActiveWatch>,
    #[cfg(test)]
    last_outcome: Option<axiom_test_support::WatchOutcome>,
    #[cfg(test)]
    last_navigation: Option<WatchNavigation>,
}

impl WatchLearner {
    pub(crate) fn from_model(model: WatchModel) -> Self {
        Self {
            model,
            ..Self::default()
        }
    }

    pub(crate) fn focus(&mut self, focus: &DeliveryFocus, now_ms: u64) {
        let next = focused(focus);
        if same_post(self.active.as_ref(), next.as_ref()) {
            self.refresh_active(next, focus.watch_ms);
            return;
        }
        self.finish_departure(focus, now_ms);
        self.observe_navigation(focus, now_ms);
        self.active = next;
    }

    pub(crate) fn playback(&mut self, event: &DeliveryPlayback, now_ms: u64) {
        let phase = event.observation.phase();
        let Some(terminal) = self.update_active(event) else {
            return;
        };
        let Some(kind) = playback_outcome(phase).filter(|_| !terminal) else {
            return;
        };
        self.record_active(kind, now_ms);
        if phase == PlaybackPhase::Inactive {
            self.record_navigation(WatchNavigation::Exit, now_ms);
        }
    }

    fn update_active(&mut self, event: &DeliveryPlayback) -> Option<bool> {
        let position = event
            .observation
            .position()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let active = self
            .active
            .as_mut()
            .filter(|item| item.post == *event.session.post())?;
        reset_new_generation(active, event.session.generation());
        active.watched_ms = active.watched_ms.max(position);
        Some(active.terminal)
    }

    pub(crate) fn model(&self) -> &WatchModel {
        &self.model
    }

    fn finish_departure(&mut self, focus: &DeliveryFocus, now_ms: u64) {
        let kind = departure_kind(focus);
        if let Some(active) = self.active.as_mut().filter(|item| !item.terminal) {
            active.watched_ms = active.watched_ms.max(focus.watch_ms);
        }
        if self.active.as_ref().is_some_and(|item| !item.terminal) {
            self.record_active(kind, now_ms);
        }
    }

    fn observe_navigation(&mut self, focus: &DeliveryFocus, now_ms: u64) {
        if focus.transition != FocusTransition::UserNavigation {
            return;
        }
        let Some(previous) = self.active.as_ref().map(|active| &active.post) else {
            return;
        };
        if let Some(event) = navigation(previous, focus) {
            self.record_navigation(event, now_ms);
        }
    }

    fn record_active(&mut self, kind: WatchSampleKind, now_ms: u64) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        active.terminal = true;
        self.model.observe(&WatchSample::new(
            active.context.clone(),
            active.watched_ms,
            kind,
            now_ms,
        ));
        #[cfg(test)]
        {
            self.last_outcome = Some(axiom_test_support::WatchOutcome::sample(
                active.watched_ms,
                kind,
            ));
        }
    }

    fn record_navigation(&mut self, event: WatchNavigation, now_ms: u64) {
        self.model.observe_navigation(event, now_ms);
        #[cfg(test)]
        {
            self.last_navigation = Some(event);
        }
    }

    fn refresh_active(&mut self, next: Option<ActiveWatch>, watch_ms: u64) {
        if let Some(active) = self.active.as_mut() {
            active.watched_ms = active.watched_ms.max(watch_ms);
            if let Some(next) = next {
                active.context = next.context;
            }
        }
    }
}

fn reset_new_generation(active: &mut ActiveWatch, generation: u64) {
    if generation > active.generation {
        active.generation = generation;
        active.terminal = false;
        active.watched_ms = 0;
    }
}

fn playback_outcome(phase: PlaybackPhase) -> Option<WatchSampleKind> {
    match phase {
        PlaybackPhase::Ended => Some(WatchSampleKind::Completed),
        PlaybackPhase::Failed => Some(WatchSampleKind::Censored(WatchCensor::DecodeFailure)),
        PlaybackPhase::Inactive => Some(WatchSampleKind::Abandoned),
        _ => None,
    }
}

#[cfg(test)]
#[path = "watch_axiom_test.rs"]
pub(crate) mod axiom_test_support;
