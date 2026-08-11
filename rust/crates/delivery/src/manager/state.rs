//! The delivery manager's owned engine state: catalog, focus, budget,
//! and the inventory control loop. Pure bookkeeping — no IO.

use crate::candidate_priority::CandidatePriority;
use crate::delivery_events::{DeliveryCandidate, FocusGenerationGuard};
use ghostr_engine::budget::params_for;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::focus::{FocusState, FocusUpdate};
use ghostr_engine::inventory_controller::{
    InventoryController, InventoryState, Mode, PresentRanges,
};
use ghostr_engine::playback::PlaybackStatus;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId};
use std::collections::HashSet;
use tokio::sync::watch;

pub(crate) struct DeliveryState {
    catalog: Catalog,
    focus: FocusState,
    controller: InventoryController,
    base: EngineParams,
    effective: EngineParams,
    level: DataUsageLevel,
    mode_watch: Option<watch::Sender<Mode>>,
    candidates: CandidatePriority,
    projection_focus: bool,
    playback: PlaybackStatus,
    focus_generations: FocusGenerationGuard,
    pending_representations: Vec<RepresentationBinding>,
}

mod focus;
mod playback;
mod representation;

impl DeliveryState {
    pub(crate) fn new(base: EngineParams, level: DataUsageLevel) -> Self {
        let effective = params_for(level, base);
        Self {
            catalog: Catalog::new(),
            focus: FocusState::new(),
            controller: InventoryController::new(effective),
            base,
            effective,
            level,
            mode_watch: None,
            candidates: CandidatePriority::default(),
            projection_focus: true,
            playback: PlaybackStatus::default(),
            focus_generations: FocusGenerationGuard::default(),
            pending_representations: Vec::new(),
        }
    }

    /// Admits validated relay output independently of any UI round trip.
    /// Until a consumer supplies focus, newest candidates form the
    /// initial priority window and begin probing/downloading immediately.
    pub(crate) fn apply_candidate(&mut self, candidate: DeliveryCandidate) {
        if candidate.meta.delivery != DeliveryKind::Progressive {
            return;
        }
        self.upsert_progressive(candidate.post.clone(), candidate.meta);
        self.candidates
            .rank(candidate.post, candidate.discovered_at);
        if self.projection_focus {
            self.focus.update_focus(FocusUpdate {
                window: self.candidates.ranked(),
                current_index: 0,
                watch_ms: 0,
            });
        }
        self.prune_scheduling_state();
    }

    pub(crate) fn prioritize(&mut self, post: PostId) {
        let mut window = self.candidate_posts();
        window.retain(|candidate| candidate != &post);
        window.insert(0, post);
        self.projection_focus = false;
        self.focus.update_focus(FocusUpdate {
            window,
            current_index: 0,
            watch_ms: 0,
        });
        self.discard_inactive_playback();
        self.prune_scheduling_state();
    }

    /// Publishes inventory mode transitions to `sender` (unified
    /// control loop, plan §5.4): discovery widens on hunger and stays
    /// quiet in comfort. Only actual transitions notify receivers.
    pub(crate) fn publish_modes(&mut self, sender: watch::Sender<Mode>) {
        self.mode_watch = Some(sender);
    }

    /// Re-derives the budgeted parameters from the pristine base
    /// (never from an already-scaled result).
    pub(crate) fn apply_level(&mut self, level: DataUsageLevel) {
        self.level = level;
        self.effective = params_for(level, self.base);
        self.controller = InventoryController::new(self.effective);
    }

    pub(crate) fn clear(&mut self) {
        self.catalog = Catalog::new();
        self.focus = FocusState::new();
        self.controller = InventoryController::new(self.effective);
        self.candidates = CandidatePriority::default();
        self.projection_focus = true;
        self.playback.discard_session();
        self.pending_representations.clear();
    }

    pub(crate) fn observe_inventory(
        &mut self,
        present: &PresentRanges,
        head_seconds: &dyn Fn(&PostId) -> u64,
    ) -> InventoryState {
        let state =
            self.controller
                .inventory_state_with(&self.catalog, &self.focus, present, head_seconds);
        self.notify_mode(state.mode);
        state
    }

    fn notify_mode(&self, mode: Mode) {
        let Some(sender) = &self.mode_watch else {
            return;
        };
        sender.send_if_modified(|current| {
            let changed = *current != mode;
            if changed {
                *current = mode;
            }
            changed
        });
    }

    pub(crate) fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub(crate) fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    pub(crate) fn focus(&self) -> &FocusState {
        &self.focus
    }

    pub(crate) fn params(&self) -> &EngineParams {
        &self.effective
    }

    pub(crate) fn concurrency(&self) -> usize {
        self.effective.concurrency(self.level)
    }

    /// Window posts in scroll order, deduplicated.
    pub(crate) fn window_posts(&self) -> Vec<PostId> {
        let mut seen = HashSet::new();
        self.focus
            .window()
            .iter()
            .filter(|post| seen.insert((*post).clone()))
            .cloned()
            .collect()
    }

    /// Probe order is mutable: focused posts first, then every other
    /// admitted candidate by newest discovery rank.
    pub(crate) fn candidate_posts(&self) -> Vec<PostId> {
        let mut posts = self.window_posts();
        let mut seen: HashSet<_> = posts.iter().cloned().collect();
        posts.extend(
            self.candidates
                .ranked()
                .into_iter()
                .filter(|post| seen.insert(post.clone())),
        );
        posts
    }
}
