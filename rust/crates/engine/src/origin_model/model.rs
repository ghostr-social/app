use super::circuit::{CircuitBook, CircuitStatus};
use super::estimate::build_estimate;
use super::exploration::ExplorationBudget;
use super::hierarchy::aggregate;
use super::keys::{OriginContextKey, OriginMethodKey, UrlContextKey};
use super::record::AdaptiveRecord;
use super::retention::retain_oldest;
use super::{
    ColdStartPrior, ColdStartSelector, DecisionMode, ModelTiming, OriginEstimate,
    OriginObservation, OriginOutcome, OriginQuery, PriorRegistration,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod replay;

const GLOBAL_CAP: usize = 128;
const ORIGIN_CAP: usize = 384;
const URL_CAP: usize = 768;
const EXPLORATION_SAMPLES: f64 = 8.0;
const SPARSE_PROBE_BYTES: u64 = 65_536;
const PRIOR_CAP: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admission {
    Production,
    Exploration { maximum_bytes: u64 },
    RecoveryProbe { maximum_bytes: u64 },
    Blocked,
}

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
    pub fn register_cold_start(&mut self, selector: ColdStartSelector, prior: ColdStartPrior) {
        if self.priors.len() == PRIOR_CAP {
            self.priors.remove(0);
        }
        self.priors.push(PriorRegistration { selector, prior });
    }

    pub(crate) fn normalize_loaded(&mut self) {
        retain_oldest(&mut self.global, GLOBAL_CAP);
        retain_oldest(&mut self.origins, ORIGIN_CAP);
        retain_oldest(&mut self.urls, URL_CAP);
        self.circuits.normalize_loaded();
        let excess = self.priors.len().saturating_sub(PRIOR_CAP);
        self.priors.drain(..excess);
    }

    pub fn observe(&mut self, item: OriginObservation) {
        if item.outcome == OriginOutcome::Cancelled {
            return;
        }
        let timing = ModelTiming::default();
        self.global
            .entry(item.query.context)
            .or_default()
            .observe(&item, timing);
        let origin = origin_key(&item.query);
        self.origins
            .entry(origin)
            .or_default()
            .observe(&item, timing);
        let url = url_key(&item.query);
        self.urls.entry(url).or_default().observe(&item, timing);
        self.observe_circuit(&item);
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

    pub fn claim(&mut self, query: &OriginQuery, now: u64, mode: DecisionMode) -> Admission {
        let key = circuit_key(query);
        match self.circuits.status(&key, now) {
            CircuitStatus::Open => return Admission::Blocked,
            CircuitStatus::Recovery => {
                return match self.circuits.claim(key, now) {
                    true => Admission::RecoveryProbe {
                        maximum_bytes: SPARSE_PROBE_BYTES,
                    },
                    false => Admission::Blocked,
                };
            }
            CircuitStatus::Closed => {}
        }
        let samples = self.estimate(query, now, mode).effective_samples;
        if mode != DecisionMode::Normal || samples >= EXPLORATION_SAMPLES {
            return Admission::Production;
        }
        match self.exploration.claim(query.origin(), now) {
            true => Admission::Exploration {
                maximum_bytes: SPARSE_PROBE_BYTES,
            },
            false => Admission::Blocked,
        }
    }

    pub fn circuit_admission(&self, query: &OriginQuery, now: u64) -> Admission {
        match self.circuits.status(&circuit_key(query), now) {
            CircuitStatus::Closed => Admission::Production,
            CircuitStatus::Open => Admission::Blocked,
            CircuitStatus::Recovery => Admission::RecoveryProbe {
                maximum_bytes: SPARSE_PROBE_BYTES,
            },
        }
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
        let success = match item.outcome {
            OriginOutcome::Success => item.range_compliant != Some(false),
            OriginOutcome::Failure(_) => false,
            OriginOutcome::Cancelled => return,
        };
        self.circuits
            .observe(circuit_key(&item.query), success, item.observed_at_ms);
    }
}

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
