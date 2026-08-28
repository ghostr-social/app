use super::Builder;
use crate::adaptive::CandidateSnapshot;
use crate::origin_model::{
    OriginAdmissionIntent, OriginContext, OriginQuery, OriginRequestProfile,
};

impl Builder<'_> {
    pub(in crate::adaptive::warp::generation) fn permits_request(
        &self,
        candidate: &CandidateSnapshot,
    ) -> bool {
        self.context.permits_request(&candidate.post)
    }

    pub(in crate::adaptive::warp::generation) fn admitted_request_source<'a>(
        &self,
        candidate: &'a CandidateSnapshot,
        action: &super::super::super::ActionKind,
    ) -> Option<&'a str> {
        self.admitted_source(candidate, action, OriginAdmissionIntent::Delivery)
    }

    pub(in crate::adaptive::warp::generation) fn optional_exploration_source<'a>(
        &self,
        candidate: &'a CandidateSnapshot,
        action: &super::super::super::ActionKind,
    ) -> Option<&'a str> {
        self.admitted_source(
            candidate,
            action,
            OriginAdmissionIntent::OptionalExploration,
        )
    }

    fn admitted_source<'a>(
        &self,
        candidate: &'a CandidateSnapshot,
        action: &super::super::super::ActionKind,
        intent: OriginAdmissionIntent,
    ) -> Option<&'a str> {
        if self.generation_policies.origin_admission
            == super::super::OriginAdmissionGenerationPolicy::LegacyUnclassified
        {
            return self
                .permits_request(candidate)
                .then(|| crate::adaptive::sources::best_origin(candidate))
                .flatten()
                .map(|origin| origin.source.as_str());
        }
        self.permits_request(candidate)
            .then(|| self.best_admitted_origin(candidate, action, intent))
            .flatten()
            .map(|origin| origin.source.as_str())
    }

    fn best_admitted_origin<'a>(
        &self,
        candidate: &'a CandidateSnapshot,
        action: &super::super::super::ActionKind,
        intent: OriginAdmissionIntent,
    ) -> Option<&'a crate::adaptive::OriginHealth> {
        crate::adaptive::sources::best_origin_where(candidate, |origin| {
            self.source_admitted(candidate, action, &origin.source, intent)
        })
    }

    pub(in crate::adaptive::warp::generation) fn source_admitted(
        &self,
        candidate: &CandidateSnapshot,
        action: &super::super::super::ActionKind,
        source: &str,
        intent: OriginAdmissionIntent,
    ) -> bool {
        if self.generation_policies.origin_admission
            == super::super::OriginAdmissionGenerationPolicy::LegacyUnclassified
        {
            return true;
        }
        let Some(profile) = super::super::request_profile::for_action(candidate, action) else {
            return true;
        };
        let query = OriginQuery::new(source, self.origin_context(source, profile));
        self.origins
            .admission_block_reason(
                &query,
                self.snapshot.observed_at_ms,
                super::super::prediction::decision_mode(self.base.mode),
                intent,
            )
            .is_none()
    }

    fn origin_context(&self, source: &str, profile: OriginRequestProfile) -> OriginContext {
        profile
            .context()
            .with_concurrency(
                self.context
                    .request_occupancy()
                    .authority_count(source)
                    .saturating_add(1),
            )
            .with_network(self.context.network_class())
            .with_observed_at_ms(self.snapshot.observed_at_ms)
    }
}
