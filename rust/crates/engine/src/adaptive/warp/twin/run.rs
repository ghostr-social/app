use super::summary::{summarize, ParticleOutcome};
use super::{TwinConfig, TwinEpochs, TwinEvaluation, TwinState, TwinStateSignature};
use crate::adaptive::{ActionNode, ResourcePrices};
use crate::PostId;
use std::collections::BTreeSet;

mod timing;

#[derive(Clone, Copy)]
pub(super) struct SimulationRun {
    state: TwinState,
    config: TwinConfig,
    prices: ResourcePrices,
    epochs: TwinEpochs,
}

impl SimulationRun {
    pub(super) const fn new(
        state: TwinState,
        config: TwinConfig,
        prices: ResourcePrices,
        epochs: TwinEpochs,
    ) -> Self {
        Self {
            state,
            config,
            prices,
            epochs,
        }
    }
}

pub(super) fn simulate(run: SimulationRun, actions: &[ActionNode]) -> TwinEvaluation {
    let seed = common_seed(run.state.signature(), run.epochs);
    let count = usize::from(run.config.particles.max(1));
    let mut outcomes: Vec<_> = (0..count)
        .map(|particle| particle_outcome(run, actions, seed, particle as u64))
        .collect();
    outcomes.sort_by_key(|item| item.delay_ms);
    summarize(&outcomes, run.config.tail_bps, seed)
}

pub(super) fn common_seed(state: TwinStateSignature, epochs: TwinEpochs) -> u64 {
    mix(state.0 ^ epochs.evidence ^ epochs.model.rotate_left(13) ^ epochs.budget.rotate_left(29))
}

fn particle_outcome(
    run: SimulationRun,
    actions: &[ActionNode],
    seed: u64,
    particle: u64,
) -> ParticleOutcome {
    let mut result = ParticleState::new(run.state);
    for (index, action) in actions.iter().enumerate() {
        let sample = sample(
            seed ^ mix(index as u64),
            particle,
            action,
            run.state.forward_swipes_per_minute,
        );
        result.apply(run, action, sample);
    }
    result.finish(run.state)
}

struct ParticleState {
    elapsed_ms: u64,
    coverage_ms: u64,
    cache_bytes: u64,
    score: i64,
    quality_posts: BTreeSet<PostId>,
}

impl ParticleState {
    fn new(state: TwinState) -> Self {
        Self {
            elapsed_ms: 0,
            coverage_ms: state.ready_coverage_ms,
            cache_bytes: state.cache_bytes,
            score: 0,
            quality_posts: BTreeSet::new(),
        }
    }

    fn apply(&mut self, run: SimulationRun, action: &ActionNode, sample: Sample) {
        self.elapsed_ms = self.elapsed_ms.saturating_add(timing::completion_ms(
            run.state,
            action,
            sample.quantile_bps,
            sample.success,
        ));
        self.score = self
            .score
            .saturating_add(action.value.total(action.resources, run.prices));
        if sample.success {
            self.apply_success(action, sample);
        }
    }

    fn apply_success(&mut self, action: &ActionNode, sample: Sample) {
        self.coverage_ms = self.coverage_ms.saturating_add(
            action
                .forecast
                .ready_playback_ms
                .min(sample.watch_duration_ms),
        );
        self.credit_quality(action);
        if sample.cache_reused {
            self.cache_bytes = self
                .cache_bytes
                .saturating_add(action.resources.storage_bytes);
        }
    }

    fn credit_quality(&mut self, action: &ActionNode) {
        let gain = action.forecast.quality_gain_micros;
        if gain > 0 && self.quality_posts.insert(action.post.clone()) {
            self.score = self.score.saturating_add(gain as i64);
        }
    }

    fn finish(self, state: TwinState) -> ParticleOutcome {
        let delay_ms = self.elapsed_ms.saturating_sub(state.current_buffer_ms);
        ParticleOutcome {
            score: self
                .score
                .saturating_sub(as_i64(delay_ms.saturating_mul(1_000))),
            delay_ms,
            coverage_ms: self.coverage_ms,
            cache_bytes: self.cache_bytes,
        }
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
    let value = mix(seed ^ mix(particle));
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

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

fn as_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}
