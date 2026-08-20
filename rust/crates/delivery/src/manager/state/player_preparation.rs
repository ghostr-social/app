use super::DeliveryState;
use crate::client_capability::{
    CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal,
    ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus,
};
use crate::delivery_events::{PlayerPreparationReport, PlayerPreparationState};
use ghostr_engine::adaptive::{PlannerCapability, PlayerPreparation};
use ghostr_engine::evidence::{EvidenceField, EvidenceValue};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::HashMap;

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
        let Some(binding) = self.catalog.binding(post) else {
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
        PlannerCapability::reported(supported, None, self.client_capabilities.revision())
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
        self.player_preparations.retain(|post, report| {
            allowed.contains(post) && catalog.binding(post).as_ref() == Some(report.binding())
        });
    }

    fn player_authority_is_current(&self, report: &PlayerPreparationReport) -> bool {
        self.demand_posts().contains(report.post())
            && self.catalog.binding(report.post()).as_ref() == Some(report.binding())
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

fn capability_signal(report: &PlayerPreparationReport) -> Option<CapabilitySignal> {
    match report.state() {
        PlayerPreparationState::Initializing => Some(CapabilitySignal::Initializing),
        PlayerPreparationState::FirstFrameRendered => Some(CapabilitySignal::FirstFrameRendered),
        PlayerPreparationState::Released => Some(CapabilitySignal::Released),
        PlayerPreparationState::Failed if report.failure_kind() == Some("invalidVideoTrack") => {
            Some(CapabilitySignal::UnsupportedFailure)
        }
        PlayerPreparationState::Failed => Some(CapabilitySignal::InconclusiveFailure),
        PlayerPreparationState::Initialized => None,
    }
}

fn codec(assessment: &ghostr_engine::evidence::EvidenceAssessment) -> Option<&str> {
    match assessment.value(EvidenceField::Codec) {
        Some(EvidenceValue::Codec(value)) => Some(value),
        _ => None,
    }
}

fn dimensions(assessment: &ghostr_engine::evidence::EvidenceAssessment) -> Option<(u32, u32)> {
    match assessment.value(EvidenceField::Dimensions) {
        Some(EvidenceValue::Dimensions { width, height }) => Some((*width, *height)),
        _ => None,
    }
}
