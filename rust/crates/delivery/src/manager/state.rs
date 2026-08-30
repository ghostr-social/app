//! Manager-owned catalog, focus, budget, and inventory state. Pure bookkeeping — no IO.

use crate::candidate_priority::CandidatePriority;
use crate::client_capability::ClientCapabilityModel;
use crate::delivery_events::{
    DeliveryCandidate, FocusGenerationGuard, PlaybackPresentation, PlayerPreparationReport,
};
use ghostr_engine::adaptive::{CurrentAuthority, DiscoveryDemand, NavigationHistory};
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
    provisional_focus_handoff: provisional_handoff::ProvisionalFocusHandoff,
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
    latest_player_client_epoch: u64,
    client_capabilities: ClientCapabilityModel,
    transform_profile: Option<crate::transform::TransformProfile>,
    fast_start_evidence: HashMap<PostId, fast_start::FastStartEvidence>,
    active_transforms: std::collections::HashSet<PostId>,
    transformed_posts: HashMap<PostId, RepresentationBinding>,
    observation_posts: HashSet<PostId>,
    ready_target: usize,
    network_status: crate::delivery_events::DeliveryNetworkStatus,
    focus_generation: crate::delivery_events::FocusGeneration,
    network_profile_generation: u64,
}

mod access;
mod evictions;
mod fast_start;
mod focus;
pub(crate) mod network;
mod playback;
mod playback_evidence;
mod player_preparation;
mod presentation;
mod provisional_handoff;
pub(crate) use presentation::PresentationAdmission;
#[cfg(test)]
#[path = "state/probes_axiom_test.rs"]
mod probes_test;
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
            provisional_focus_handoff: Default::default(),
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
            latest_player_client_epoch: 0,
            client_capabilities: ClientCapabilityModel::default(),
            transform_profile: None,
            fast_start_evidence: Default::default(),
            active_transforms: Default::default(),
            transformed_posts: Default::default(),
            observation_posts: Default::default(),
            ready_target: 1,
            network_status: crate::delivery_events::DeliveryNetworkStatus::unavailable(),
            focus_generation: crate::delivery_events::FocusGeneration::compatibility(),
            network_profile_generation: 0,
        }
    }

    /// Admits relay output while keeping the projected current pinned.
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
    pub(super) fn apply_level(&mut self, level: DataUsageLevel) {
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
        self.provisional_focus_handoff.clear();
        self.playback.discard_session();
        self.pending_presentation = None;
        self.hls_focus.clear();
        self.pending_representations.clear();
        self.changed_representations.clear();
        self.navigation = NavigationHistory::default();
        self.recent_evictions.clear();
        self.prune_player_preparations(&HashMap::new());
        self.latest_player_client_epoch = 0;
        self.fast_start_evidence.clear();
        self.active_transforms.clear();
        self.transformed_posts.clear();
        self.observation_posts.clear();
        self.ready_target = 1;
        self.focus_generation = crate::delivery_events::FocusGeneration::compatibility();
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
}
