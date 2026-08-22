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
    let mut posts = BTreeMap::new();
    for action in actions {
        let next = posts.len().min(usize::from(u16::MAX)) as u16;
        let post = *posts.entry(action.post.as_str()).or_insert(next);
        post.hash(&mut hasher);
        stable_hash(action, &mut hasher);
    }
    hasher.finish()
}

fn stable_hash(action: &ActionNode, state: &mut impl Hasher) {
    action.id.hash(state);
    stable_kind(&action.kind, state);
    action.value.hash(state);
    action.resources.hash(state);
    action.forecast.hash(state);
    action.requires.hash(state);
    action.request().hash(state);
}

fn stable_kind(kind: &crate::adaptive::ActionKind, state: &mut impl Hasher) {
    use crate::adaptive::ActionKind;
    std::mem::discriminant(kind).hash(state);
    match kind {
        ActionKind::Prefix(range)
        | ActionKind::Tail(range)
        | ActionKind::FetchRange(range)
        | ActionKind::CacheUpgrade(range) => range.hash(state),
        ActionKind::FetchWhole { maximum_bytes } => maximum_bytes.hash(state),
        ActionKind::HlsBootstrap {
            stage,
            maximum_bytes,
        } => (stage, maximum_bytes).hash(state),
        ActionKind::Promote {
            active,
            maximum_bytes,
        } => (active, maximum_bytes).hash(state),
        ActionKind::Transform(value) => value.hash(state),
        ActionKind::Hedge { primary, .. } | ActionKind::Cancel(primary) => primary.hash(state),
        ActionKind::Head => {}
    }
}
