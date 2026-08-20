use super::summary::{summarize, ParticleOutcome};
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

    pub fn evaluate(
        &mut self,
        state: &TwinState,
        actions: &[ActionNode],
        epochs: TwinEpochs,
    ) -> TwinEvaluation {
        let signature = state.signature();
        let key = cache_key(signature, actions, epochs);
        if let Some(cached) = self.cache.get(&key) {
            return *cached;
        }
        let result = simulate(*state, actions, self.config, self.prices, epochs);
        self.cache.insert(key, result);
        result
    }

    pub fn cache_entries(&self) -> usize {
        self.cache.len()
    }
}

fn simulate(
    state: TwinState,
    actions: &[ActionNode],
    config: TwinConfig,
    prices: ResourcePrices,
    epochs: TwinEpochs,
) -> TwinEvaluation {
    let seed = common_seed(state.signature(), epochs);
    let count = usize::from(config.particles.max(1));
    let mut outcomes: Vec<_> = (0..count)
        .map(|particle| particle_outcome(state, actions, prices, seed, particle as u64))
        .collect();
    outcomes.sort_by_key(|item| item.delay_ms);
    summarize(&outcomes, config.tail_bps, seed)
}

fn particle_outcome(
    state: TwinState,
    actions: &[ActionNode],
    prices: ResourcePrices,
    seed: u64,
    particle: u64,
) -> ParticleOutcome {
    let mut elapsed = 0_u64;
    let mut coverage = state.ready_coverage_ms;
    let mut cache = state.cache_bytes;
    let mut score = 0_i64;
    for (index, action) in actions.iter().enumerate() {
        let sample = sample(
            seed ^ mix(index as u64),
            particle,
            action,
            state.forward_swipes_per_minute,
        );
        elapsed = elapsed.saturating_add(completion_ms(state, action, sample));
        score = score.saturating_add(action.value.total(action.resources, prices));
        if sample.success {
            coverage = coverage.saturating_add(
                action
                    .forecast
                    .ready_playback_ms
                    .min(sample.watch_duration_ms),
            );
            score = score.saturating_add(action.forecast.quality_gain_micros as i64);
            if sample.cache_reused {
                cache = cache.saturating_add(action.resources.storage_bytes);
            }
        }
    }
    let delay = elapsed.saturating_sub(state.current_buffer_ms);
    ParticleOutcome {
        score: score.saturating_sub(as_i64(delay.saturating_mul(1_000))),
        delay_ms: delay,
        coverage_ms: coverage,
        cache_bytes: cache,
    }
}

#[derive(Clone, Copy)]
struct Sample {
    quantile_bps: u16,
    success: bool,
    cache_reused: bool,
    watch_duration_ms: u64,
}

fn sample(seed: u64, particle: u64, action: &ActionNode, swipe_rate: u16) -> Sample {
    let origin = text_hash(&action.origin);
    let value = mix(seed ^ mix(particle) ^ origin);
    Sample {
        quantile_bps: (value % 10_000) as u16,
        success: ((value >> 16) % 10_000) < u64::from(action.forecast.success_bps),
        cache_reused: ((value >> 32) % 10_000) < u64::from(action.forecast.cache_reuse_bps),
        watch_duration_ms: sampled_watch_ms(
            value >> 48,
            swipe_rate,
            action.forecast.ready_playback_ms,
        ),
    }
}

fn sampled_watch_ms(random: u64, swipe_rate: u16, maximum_ms: u64) -> u64 {
    let mean = match swipe_rate {
        0 => maximum_ms,
        rate => 60_000 / u64::from(rate),
    };
    let centered_bps = 5_000_u64.saturating_add(random % 10_001);
    mean.saturating_mul(centered_bps)
        .saturating_div(10_000)
        .min(maximum_ms)
}

fn completion_ms(state: TwinState, action: &ActionNode, sample: Sample) -> u64 {
    let times = action.forecast.completion;
    let fallback = state.rtt_ms.saturating_add(
        action.resources.network_bytes.saturating_mul(8_000) / state.throughput_bps.max(1),
    );
    let expected = times.expected_ms.max(fallback);
    let sampled = interpolate(
        expected,
        times.p95_ms.max(expected),
        times.p99_ms.max(expected),
        sample.quantile_bps,
    );
    if sample.success && action.resources.requests <= state.request_slots {
        sampled
    } else {
        sampled.max(times.cvar_ms).saturating_mul(2)
    }
}

fn interpolate(expected: u64, p95: u64, p99: u64, quantile: u16) -> u64 {
    match quantile {
        0..=4_999 => expected.saturating_mul(50 + u64::from(quantile) / 100) / 100,
        5_000..=9_499 => expected + (p95 - expected) * u64::from(quantile - 5_000) / 4_500,
        _ => p95 + (p99 - p95) * u64::from(quantile - 9_500) / 500,
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

fn common_seed(state: TwinStateSignature, epochs: TwinEpochs) -> u64 {
    mix(state.0 ^ epochs.evidence ^ epochs.model.rotate_left(13) ^ epochs.budget.rotate_left(29))
}

fn text_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

fn as_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}
