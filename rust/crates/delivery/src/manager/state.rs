//! The delivery manager's owned engine state: catalog, focus, budget,
//! and the inventory control loop. Pure bookkeeping — no IO.

use crate::candidate_priority::CandidatePriority;
use crate::delivery_events::{DeliveryCandidate, FocusGenerationGuard};
use ghostr_engine::adaptive::{
    CurrentAuthority, DiscoveryDemand, NavigationHistory, NavigationSnapshot,
};
use ghostr_engine::budget::params_for;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::focus::{FocusState, FocusUpdate};
use ghostr_engine::playback::PlaybackStatus;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId};
use std::collections::HashMap;
use tokio::sync::watch;

pub(crate) struct DeliveryState {
    catalog: Catalog,
    focus: FocusState,
    base: EngineParams,
    effective: EngineParams,
    level: DataUsageLevel,
    discovery_watch: Option<watch::Sender<DiscoveryDemand>>,
    candidates: CandidatePriority,
    current_authority: CurrentAuthority,
    playback: PlaybackStatus,
    focus_generations: FocusGenerationGuard,
    pending_representations: Vec<RepresentationBinding>,
    changed_representations: Vec<PostId>,
    navigation: NavigationHistory,
    recent_evictions: HashMap<PostId, Vec<ghostr_engine::ByteRange>>,
}

mod evictions;
mod focus;
mod playback;
mod probes;
mod representation;
mod window;

impl DeliveryState {
    pub(crate) fn new(base: EngineParams, level: DataUsageLevel) -> Self {
        let effective = params_for(level, base);
        Self {
            catalog: Catalog::new(),
            focus: FocusState::new(),
            base,
            effective,
            level,
            discovery_watch: None,
            candidates: CandidatePriority::default(),
            current_authority: CurrentAuthority::Provisional,
            playback: PlaybackStatus::default(),
            focus_generations: FocusGenerationGuard::default(),
            pending_representations: Vec::new(),
            changed_representations: Vec::new(),
            navigation: NavigationHistory::default(),
            recent_evictions: HashMap::new(),
        }
    }

    /// Admits validated relay output independently of any UI round trip.
    /// Until a consumer supplies focus, candidates form the initial
    /// priority window and begin probing/downloading immediately. The
    /// projected current post stays pinned across arrivals: the feed
    /// accumulates rows, so its top post is the first one served, and
    /// re-aiming startup work at every newer discovery restarts the
    /// first video's delivery over and over.
    pub(crate) fn apply_candidate(&mut self, candidate: DeliveryCandidate) {
        if candidate.meta.delivery != DeliveryKind::Progressive {
            return;
        }
        self.upsert_progressive_renditions(
            candidate.post.clone(),
            candidate.meta,
            candidate.renditions,
        );
        self.candidates
            .rank(candidate.post, candidate.discovered_at);
        if self.current_authority == CurrentAuthority::Provisional {
            let window = self.candidates.ranked();
            let current_index = self
                .focus
                .current()
                .and_then(|current| window.iter().position(|post| post == current))
                .unwrap_or(0);
            self.focus.update_focus(FocusUpdate {
                window,
                current_index,
                watch_ms: 0,
            });
        }
        self.prune_scheduling_state();
    }

    pub(crate) fn publish_discovery_demand(&mut self, sender: watch::Sender<DiscoveryDemand>) {
        self.discovery_watch = Some(sender);
    }

    /// Re-derives the budgeted parameters from the pristine base
    /// (never from an already-scaled result).
    pub(crate) fn apply_level(&mut self, level: DataUsageLevel) {
        self.level = level;
        self.effective = params_for(level, self.base);
    }

    pub(crate) fn clear(&mut self) {
        self.catalog = Catalog::new();
        self.focus = FocusState::new();
        self.candidates = CandidatePriority::default();
        self.current_authority = CurrentAuthority::Provisional;
        self.playback.discard_session();
        self.pending_representations.clear();
        self.changed_representations.clear();
        self.navigation = NavigationHistory::default();
        self.recent_evictions.clear();
    }

    pub(crate) fn observe_discovery_demand(&self, demand: DiscoveryDemand) {
        let Some(sender) = &self.discovery_watch else {
            return;
        };
        sender.send_if_modified(|current| {
            let changed = *current != demand;
            if changed {
                *current = demand;
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

    pub(crate) fn current_authority(&self) -> CurrentAuthority {
        self.current_authority
    }

    pub(crate) fn params(&self) -> &EngineParams {
        &self.effective
    }

    pub(crate) fn concurrency(&self) -> usize {
        self.effective.concurrency(self.level)
    }

    pub(crate) fn navigation(&self, observed_at_ms: u64) -> NavigationSnapshot {
        self.navigation.snapshot(observed_at_ms)
    }
}
