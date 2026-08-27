use super::circuit::CircuitBook;
use super::estimate::build_estimate;
use super::exploration::ExplorationBudget;
use super::hierarchy::aggregate;
use super::keys::{OriginContextKey, OriginMethodKey, UrlContextKey};
use super::record::AdaptiveRecord;
use super::retention::retain_oldest;
use super::{
    ColdStartPrior, DecisionMode, ModelTiming, OriginEstimate, OriginObservation, OriginOutcome,
    OriginQuery, PriorRegistration,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod claim;
mod replay;
pub use claim::{Admission, AdmissionClaim, AdmissionClaimTerminal, ClaimedAdmission};

const GLOBAL_CAP: usize = 128;
const ORIGIN_CAP: usize = 384;
const URL_CAP: usize = 768;
const EXPLORATION_SAMPLES: f64 = 8.0;
const SPARSE_PROBE_BYTES: u64 = 65_536;
const PRIOR_CAP: usize = 128;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OriginModel {
    #[serde(default, with = "super::map_serde")]
    global: BTreeMap<super::OriginContext, AdaptiveRecord>,
    #[serde(default, with = "super::map_serde")]
    origins: BTreeMap<OriginContextKey, AdaptiveRecord>,
    #[serde(default, with = "super::map_serde")]
    urls: BTreeMap<UrlContextKey, AdaptiveRecord>,
    #[serde(default)]
    circuits: CircuitBook,
    #[serde(default)]
    priors: Vec<PriorRegistration>,
    #[serde(default)]
    exploration: ExplorationBudget,
}

impl OriginModel {
    pub(crate) fn normalize_loaded(&mut self) {
        retain_oldest(&mut self.global, GLOBAL_CAP);
        retain_oldest(&mut self.origins, ORIGIN_CAP);
        retain_oldest(&mut self.urls, URL_CAP);
        self.circuits.normalize_loaded();
        let excess = self.priors.len().saturating_sub(PRIOR_CAP);
        self.priors.drain(..excess);
    }

    pub fn observe(&mut self, item: &OriginObservation) {
        if item.outcome == OriginOutcome::Cancelled {
            return;
        }
        self.observe_records(item);
        self.observe_circuit(item);
    }

    fn observe_records(&mut self, item: &OriginObservation) {
        let timing = ModelTiming::default();
        self.global
            .entry(item.query.context)
            .or_default()
            .observe(item, timing);
        let origin = origin_key(&item.query);
        self.origins
            .entry(origin)
            .or_default()
            .observe(item, timing);
        let url = url_key(&item.query);
        self.urls.entry(url).or_default().observe(item, timing);
        retain_oldest(&mut self.global, GLOBAL_CAP);
        retain_oldest(&mut self.origins, ORIGIN_CAP);
        retain_oldest(&mut self.urls, URL_CAP);
    }

    pub fn estimate(&self, query: &OriginQuery, now: u64, mode: DecisionMode) -> OriginEstimate {
        let prior = self.prior(query);
        let records = [
            self.global.get(&query.context),
            self.origins.get(&origin_key(query)),
            self.urls.get(&url_key(query)),
        ];
        let snapshot = aggregate(records, prior, now, ModelTiming::default());
        build_estimate(
            query.context,
            query.environment.clone(),
            snapshot,
            prior,
            mode,
        )
    }

    fn prior(&self, query: &OriginQuery) -> ColdStartPrior {
        self.priors
            .iter()
            .filter_map(|item| item.selector.score(query).map(|score| (score, item.prior)))
            .max_by_key(|item| item.0)
            .map_or_else(
                || ColdStartPrior::bootstrap(query.context.method),
                |item| item.1,
            )
    }

    fn observe_circuit(&mut self, item: &OriginObservation) {
        let Some((success, observed_at_ms)) = circuit_result(item) else {
            return;
        };
        self.circuits
            .observe(circuit_key(&item.query), success, observed_at_ms);
    }
}

#[cfg(test)]
#[path = "model/test_support.rs"]
mod test_support;

fn origin_key(query: &OriginQuery) -> OriginContextKey {
    OriginContextKey {
        origin: query.origin().to_owned(),
        context: query.context,
    }
}

fn url_key(query: &OriginQuery) -> UrlContextKey {
    UrlContextKey {
        url_id: query.url_id().to_owned(),
        context: query.context,
    }
}

fn circuit_key(query: &OriginQuery) -> OriginMethodKey {
    OriginMethodKey {
        origin: query.origin().to_owned(),
        method: query.context.method,
    }
}

fn circuit_result(item: &OriginObservation) -> Option<(bool, u64)> {
    let success = match item.outcome {
        OriginOutcome::Success => item.range_compliant != Some(false),
        OriginOutcome::Failure(_) => false,
        OriginOutcome::Cancelled => return None,
    };
    Some((success, item.observed_at_ms))
}
