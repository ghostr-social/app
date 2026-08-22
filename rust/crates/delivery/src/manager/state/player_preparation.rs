use super::DeliveryState;
use crate::client_capability::{
    CapabilityAttempt, CapabilityEvent, CapabilityObservation, ClientCapabilityModel,
    ClientCapabilityProfile, ClientCapabilityStatus,
};
use crate::delivery_events::PlayerPreparationReport;
use ghostr_engine::adaptive::{PlannerCapability, PlayerPreparation, TransformCapability};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::HashMap;

mod evidence;
use evidence::{capability_signal, codec, dimensions};

impl DeliveryState {
    pub(crate) fn update_ready_target(&mut self, target: usize) {
        if self.ready_target == target {
            return;
        }
        self.ready_target = target;
        self.prune_player_preparation_scope();
    }

    #[cfg(test)]
    pub(crate) fn apply_player_preparation(&mut self, report: PlayerPreparationReport) -> bool {
        self.apply_player_preparation_at(report, 0)
    }

    pub(crate) fn apply_player_preparation_at(
        &mut self,
        report: PlayerPreparationReport,
        now_ms: u64,
    ) -> bool {
        if !self.player_authority_is_current(&report) {
            return false;
        }
        if self
            .player_preparations
            .get(report.post())
            .is_some_and(|older| !report.supersedes(older))
        {
            return false;
        }
        let observation = self.capability_observation(&report, now_ms);
        self.player_preparations
            .insert(report.post().clone(), report);
        if let Some(observation) = observation {
            self.client_capabilities.observe(observation);
        }
        true
    }

    pub(crate) fn client_capability_status(
        &self,
        post: &PostId,
        generation: u64,
        now_ms: u64,
    ) -> ClientCapabilityStatus {
        let Some(binding) = self.playback_binding(post) else {
            return ClientCapabilityStatus::Unknown;
        };
        let Some(profile) = self.capability_profile(post, &binding, now_ms) else {
            return ClientCapabilityStatus::Unknown;
        };
        self.client_capabilities.status(generation, &profile)
    }

    pub(crate) fn planner_capability(&self, post: &PostId, now_ms: u64) -> PlannerCapability {
        let Some(generation) = self.client_capabilities.current_generation() else {
            return PlannerCapability::Unavailable;
        };
        let status = self.client_capability_status(post, generation, now_ms);
        let supported = match status {
            ClientCapabilityStatus::Supported { .. } => true,
            ClientCapabilityStatus::Unsupported => false,
            _ => return PlannerCapability::Unavailable,
        };
        let transform = (!supported)
            .then(|| self.recoverable_transform(post))
            .flatten()
            .map(|profile| {
                TransformCapability::new(
                    profile.kind(),
                    profile.limits().cpu_ms(),
                    profile.limits().output_bytes(),
                )
            });
        PlannerCapability::reported(supported, transform, self.client_capabilities.revision())
    }

    pub(crate) const fn client_capability_revision(&self) -> u64 {
        self.client_capabilities.revision()
    }

    pub(crate) const fn client_capabilities(&self) -> &ClientCapabilityModel {
        &self.client_capabilities
    }

    pub(crate) fn replace_client_capabilities(&mut self, model: ClientCapabilityModel) {
        self.client_capabilities = model;
    }

    pub(crate) fn player_preparation(
        &self,
        post: &PostId,
        revision: Option<ContentRevision>,
    ) -> PlayerPreparation {
        self.player_preparations
            .get(post)
            .filter(|report| self.player_authority_is_current(report))
            .filter(|report| Some(report.revision()) == revision)
            .map_or(PlayerPreparation::Unverified, |report| {
                report.engine_state()
            })
    }

    pub(crate) fn prune_player_preparations(
        &mut self,
        revisions: &HashMap<PostId, ContentRevision>,
    ) {
        self.prune_player_preparation_scope();
        self.player_preparations.retain(|post, report| {
            revisions
                .get(post)
                .is_some_and(|revision| *revision == report.revision())
        });
    }

    pub(super) fn prune_player_preparation_scope(&mut self) {
        let allowed = self.demand_posts();
        let catalog = &self.catalog;
        let transformed = &self.transformed_posts;
        self.player_preparations.retain(|post, report| {
            let binding = transformed
                .get(post)
                .cloned()
                .or_else(|| catalog.binding(post));
            allowed.contains(post) && binding.as_ref() == Some(report.binding())
        });
    }

    fn player_authority_is_current(&self, report: &PlayerPreparationReport) -> bool {
        self.demand_posts().contains(report.post())
            && self.playback_binding(report.post()).as_ref() == Some(report.binding())
    }

    fn recoverable_transform(&self, post: &PostId) -> Option<crate::transform::TransformProfile> {
        let profile = self.transform_available_for(post)?;
        let report = self.player_preparations.get(post)?;
        if !profile.trigger().allows_failure(report.failure_kind()) {
            return None;
        }
        if profile.trigger().requires_fast_start() && !self.current_fast_start_failure(post) {
            return None;
        }
        Some(profile)
    }

    fn capability_observation(
        &self,
        report: &PlayerPreparationReport,
        now_ms: u64,
    ) -> Option<CapabilityObservation> {
        let profile = self.capability_profile(report.post(), report.binding(), now_ms)?;
        let signal = capability_signal(report)?;
        Some(CapabilityObservation::new(
            report.player_capability_generation(),
            CapabilityAttempt::new(report.client_epoch(), report.attempt_generation()),
            profile,
            CapabilityEvent::new(report.observed_monotonic_us(), signal),
        ))
    }

    fn capability_profile(
        &self,
        post: &PostId,
        binding: &RepresentationBinding,
        now_ms: u64,
    ) -> Option<ClientCapabilityProfile> {
        let entry = self.catalog.lookup(post)?;
        let assessment = entry
            .meta
            .urls
            .first()
            .map(|url| entry.evidence_assessment_for(url, now_ms));
        let codec = assessment.as_ref().and_then(codec);
        let dimensions = assessment.as_ref().and_then(dimensions);
        ClientCapabilityProfile::try_new(binding.representation().fingerprint(), codec, dimensions)
            .ok()
    }
}
