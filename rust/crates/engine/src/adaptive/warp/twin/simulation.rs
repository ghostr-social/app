use super::run::{self, SimulationRun};
use super::{TwinConfig, TwinEpochs, TwinEvaluation, TwinState, TwinStateSignature};
use crate::adaptive::{ActionNode, ResourcePrices};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CacheKey {
    state: u64,
    plan: u64,
    evidence: u64,
    model: u64,
    budget: u64,
}

pub struct DigitalTwin {
    config: TwinConfig,
    prices: ResourcePrices,
    cache: BTreeMap<CacheKey, TwinEvaluation>,
}

impl DigitalTwin {
    pub fn new(config: TwinConfig) -> Self {
        Self {
            config,
            prices: ResourcePrices::default(),
            cache: BTreeMap::new(),
        }
    }

    pub fn set_prices(&mut self, prices: ResourcePrices) {
        self.prices = prices;
    }

    pub fn common_random_seed(state: &TwinState, epochs: TwinEpochs) -> u64 {
        run::common_seed(state.signature(), epochs)
    }

    pub fn evaluate(
        &mut self,
        state: &TwinState,
        actions: &[ActionNode],
        epochs: TwinEpochs,
    ) -> TwinEvaluation {
        let key = cache_key(state.signature(), actions, epochs);
        if let Some(cached) = self.cache.get(&key) {
            return *cached;
        }
        let run = SimulationRun::new(*state, self.config, self.prices, epochs);
        let result = run::simulate(run, actions);
        self.cache.insert(key, result);
        result
    }

    pub fn cache_entries(&self) -> usize {
        self.cache.len()
    }
}

fn cache_key(state: TwinStateSignature, actions: &[ActionNode], epochs: TwinEpochs) -> CacheKey {
    CacheKey {
        state: state.0,
        plan: plan_hash(actions),
        evidence: epochs.evidence,
        model: epochs.model,
        budget: epochs.budget,
    }
}

fn plan_hash(actions: &[ActionNode]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    actions.iter().for_each(|action| action.hash(&mut hasher));
    hasher.finish()
}
