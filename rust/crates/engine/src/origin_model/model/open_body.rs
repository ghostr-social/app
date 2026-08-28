use super::{origin_key, url_key, OriginModel, GLOBAL_CAP, ORIGIN_CAP, URL_CAP};
use crate::origin_model::estimate::{build_estimate, EstimateInput};
use crate::origin_model::hierarchy::aggregate;
use crate::origin_model::retention::retain_oldest;
use crate::origin_model::{
    DecisionMode, ModelTiming, OpenBodyObservation, OriginEstimate, OriginOutcome, OriginQuery,
};

impl OriginModel {
    pub fn observe_open_body(&mut self, item: &OpenBodyObservation) {
        if item.outcome == OriginOutcome::Cancelled {
            return;
        }
        let transport = item.transport_observation();
        let timing = ModelTiming::default();
        self.open_body_global
            .entry(item.query.context)
            .or_default()
            .observe(&transport, timing);
        self.open_body_origins
            .entry(origin_key(&item.query))
            .or_default()
            .observe(&transport, timing);
        self.open_body_urls
            .entry(url_key(&item.query))
            .or_default()
            .observe(&transport, timing);
        self.retain_open_body();
    }

    pub fn estimate_open_body(
        &self,
        query: &OriginQuery,
        now: u64,
        mode: DecisionMode,
    ) -> OriginEstimate {
        let prior = self.prior(query).for_open_body();
        let records = [
            self.open_body_global.get(&query.context),
            self.open_body_origins.get(&origin_key(query)),
            self.open_body_urls.get(&url_key(query)),
        ];
        let snapshot = aggregate(records, prior, now, ModelTiming::default());
        build_estimate(EstimateInput {
            context: query.context,
            environment: query.environment.clone(),
            snapshot,
            prior,
            mode,
            success_prior_evidence: prior.success_alpha + prior.success_beta,
        })
    }

    fn retain_open_body(&mut self) {
        retain_oldest(&mut self.open_body_global, GLOBAL_CAP);
        retain_oldest(&mut self.open_body_origins, ORIGIN_CAP);
        retain_oldest(&mut self.open_body_urls, URL_CAP);
    }
}
