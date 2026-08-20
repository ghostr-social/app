use super::privacy::DecisionPrivacy;
use super::types::{DecisionModelInput, ModelQuantiles, ProbabilityEstimateRecord, QuantileRecord};

const RECORD_LIMIT: usize = 64;

pub(super) fn capture(
    inputs: &[DecisionModelInput],
    privacy: &DecisionPrivacy,
) -> Vec<ModelQuantiles> {
    inputs
        .iter()
        .take(RECORD_LIMIT)
        .map(|input| ModelQuantiles {
            source_id: privacy.source(&input.source),
            success: probability(input.success_bps),
            range_compliance: probability(input.range_compliance_bps),
            ttfb_ms: quantiles(input.ttfb_ms),
            throughput_bps: quantiles(input.throughput_bps),
            effective_samples: input.effective_samples,
            adapting: input.adapting,
            uncertainty_bps: input.uncertainty_bps,
        })
        .collect()
}

fn probability(value: [u16; 4]) -> ProbabilityEstimateRecord {
    ProbabilityEstimateRecord {
        mean_bps: value[0],
        lower_bps: value[1],
        upper_bps: value[2],
        selected_bps: value[3],
    }
}

fn quantiles(value: [u64; 6]) -> QuantileRecord {
    QuantileRecord {
        p10: value[0],
        p50: value[1],
        p90: value[2],
        p95: value[3],
        p99: value[4],
        selected: value[5],
    }
}
