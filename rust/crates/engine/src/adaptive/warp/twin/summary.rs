use super::TwinEvaluation;

#[derive(Clone, Copy)]
pub(super) struct ParticleOutcome {
    pub score: i64,
    pub delay_ms: u64,
    pub coverage_ms: u64,
    pub cache_bytes: u64,
}

pub(super) fn summarize(outcomes: &[ParticleOutcome], tail_bps: u16, seed: u64) -> TwinEvaluation {
    let length = outcomes.len() as u128;
    let tail = quantile_index(outcomes.len(), tail_bps);
    let mean = |pick: fn(&ParticleOutcome) -> u64| {
        outcomes
            .iter()
            .map(|item| u128::from(pick(item)))
            .sum::<u128>()
            / length
    };
    TwinEvaluation {
        expected_score_micros: signed_mean(outcomes),
        expected_visible_delay_ms: mean(|item| item.delay_ms) as u64,
        p95_visible_delay_ms: outcomes[quantile_index(outcomes.len(), 9_500)].delay_ms,
        p99_visible_delay_ms: outcomes[quantile_index(outcomes.len(), 9_900)].delay_ms,
        cvar_visible_delay_ms: cvar(outcomes, tail),
        on_time_probability_bps: on_time(outcomes),
        expected_ready_coverage_ms: mean(|item| item.coverage_ms) as u64,
        expected_cache_bytes: mean(|item| item.cache_bytes) as u64,
        common_random_seed: seed,
    }
}

fn signed_mean(outcomes: &[ParticleOutcome]) -> i64 {
    let sum = outcomes
        .iter()
        .fold(0_i128, |sum, item| sum + i128::from(item.score));
    let mean = sum / outcomes.len() as i128;
    mean.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn cvar(outcomes: &[ParticleOutcome], start: usize) -> u64 {
    let tail = &outcomes[start..];
    let sum: u128 = tail.iter().map(|item| u128::from(item.delay_ms)).sum();
    (sum / tail.len().max(1) as u128) as u64
}

fn on_time(outcomes: &[ParticleOutcome]) -> u16 {
    let ready = outcomes.iter().filter(|item| item.delay_ms == 0).count();
    (ready.saturating_mul(10_000) / outcomes.len().max(1)) as u16
}

fn quantile_index(length: usize, bps: u16) -> usize {
    (length.saturating_sub(1) * usize::from(bps) / 10_000).min(length.saturating_sub(1))
}
