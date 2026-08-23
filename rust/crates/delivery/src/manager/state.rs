//! The delivery manager's owned engine state: catalog, focus, budget,
//! and the inventory control loop. Pure bookkeeping — no IO.

use crate::candidate_priority::CandidatePriority;
use crate::client_capability::ClientCapabilityModel;
use crate::delivery_events::{
    DeliveryCandidate, FocusGenerationGuard, PlaybackPresentation, PlayerPreparationReport,
};
use ghostr_engine::adaptive::{
    CurrentAuthority, DiscoveryDemand, NavigationHistory, NavigationSnapshot,
};
use ghostr_engine::budget::params_for;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::focus::{FocusState, FocusUpdate};
use ghostr_engine::playback::PlaybackStatus;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId};
use std::collections::{HashMap, HashSet};
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
    latest_presentation_sequence: u64,
    pending_presentation: Option<PlaybackPresentation>,
    focus_generations: FocusGenerationGuard,
    hls_focus: HashSet<PostId>,
    pending_representations: Vec<RepresentationBinding>,
    changed_representations: Vec<PostId>,
    navigation: NavigationHistory,
    recent_evictions: HashMap<PostId, Vec<ghostr_engine::ByteRange>>,
    player_preparations: HashMap<PostId, PlayerPreparationReport>,
    client_capabilities: ClientCapabilityModel,
    transform_profile: Option<crate::transform::TransformProfile>,
    fast_start_evidence: HashMap<PostId, fast_start::FastStartEvidence>,
    active_transforms: std::collections::HashSet<PostId>,
    transformed_posts: HashMap<PostId, RepresentationBinding>,
    observation_posts: HashSet<PostId>,
    ready_target: usize,
    network_status: crate::delivery_events::DeliveryNetworkStatus,
}

mod evictions;
mod fast_start;
mod focus;
pub(crate) mod network;
mod playback;
mod playback_evidence;
mod player_preparation;
mod presentation;
pub(crate) use presentation::PresentationAdmission;
mod probes;
mod representation;
mod transform;
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
            latest_presentation_sequence: 0,
            pending_presentation: None,
            focus_generations: FocusGenerationGuard::default(),
            hls_focus: HashSet::new(),
            pending_representations: Vec::new(),
            changed_representations: Vec::new(),
            navigation: NavigationHistory::default(),
            recent_evictions: HashMap::new(),
            player_preparations: HashMap::new(),
            client_capabilities: ClientCapabilityModel::default(),
            transform_profile: None,
            fast_start_evidence: Default::default(),
            active_transforms: Default::default(),
            transformed_posts: Default::default(),
            observation_posts: Default::default(),
            ready_target: 1,
            network_status: crate::delivery_events::DeliveryNetworkStatus::unavailable(),
        }
    }

    /// Admits relay output before UI focus. The projected current stays pinned
    /// so newer discoveries cannot repeatedly restart first-video delivery.
    pub(crate) fn apply_candidate(&mut self, candidate: DeliveryCandidate) {
        if candidate.meta.delivery != DeliveryKind::Progressive
            || self.hls_focus.contains(&candidate.post)
        {
            return;
        }
        let post = candidate.post.clone();
        let discovered_at = candidate.discovered_at;
        self.upsert_progressive_candidate(candidate);
        self.candidates.rank(post, discovered_at);
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
        let evidence = self.catalog.evidence_state();
        self.catalog = Catalog::new();
        self.catalog.replace_evidence_state(evidence, 0);
        self.focus = FocusState::new();
        self.candidates = CandidatePriority::default();
        self.current_authority = CurrentAuthority::Provisional;
        self.playback.discard_session();
        self.pending_presentation = None;
        self.hls_focus.clear();
        self.pending_representations.clear();
        self.changed_representations.clear();
        self.navigation = NavigationHistory::default();
        self.recent_evictions.clear();
        self.player_preparations.clear();
        self.fast_start_evidence.clear();
        self.active_transforms.clear();
        self.transformed_posts.clear();
        self.observation_posts.clear();
        self.ready_target = 1;
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

    pub(crate) fn current_post(&self) -> Option<PostId> {
        self.focus.current().cloned()
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
