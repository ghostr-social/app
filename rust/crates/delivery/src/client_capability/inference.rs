use super::types::{
    CapabilityRecord, CapabilityResult, ClientCapabilityProfile, ClientCapabilityStatus,
};
use std::collections::VecDeque;

const LATENCY_CAPACITY: usize = 16;

pub(super) fn inferred_support(
    records: &VecDeque<CapabilityRecord>,
    profile: &ClientCapabilityProfile,
) -> Option<ClientCapabilityStatus> {
    records
        .iter()
        .filter(|record| supports(&record.profile, profile))
        .filter_map(|record| supported_latency(&record.result))
        .max()
        .map(|p95_first_frame_us| ClientCapabilityStatus::Supported { p95_first_frame_us })
}

pub(super) fn merge_result(existing: &mut CapabilityResult, incoming: CapabilityResult) {
    match (existing, incoming) {
        (
            CapabilityResult::Supported { first_frame_us },
            CapabilityResult::Supported {
                first_frame_us: incoming,
            },
        ) => append_latencies(first_frame_us, incoming),
        (existing, incoming) => *existing = incoming,
    }
}

fn append_latencies(values: &mut Vec<u64>, incoming: Vec<u64>) {
    values.extend(incoming);
    if values.len() > LATENCY_CAPACITY {
        values.drain(..values.len() - LATENCY_CAPACITY);
    }
}

pub(super) fn normalize_record(mut record: CapabilityRecord) -> Option<CapabilityRecord> {
    if !record.profile.is_valid() {
        return None;
    }
    if let CapabilityResult::Supported { first_frame_us } = &mut record.result {
        if first_frame_us.is_empty() {
            return None;
        }
        trim_latencies(first_frame_us);
    }
    Some(record)
}

fn trim_latencies(values: &mut Vec<u64>) {
    if values.len() > LATENCY_CAPACITY {
        values.drain(..values.len() - LATENCY_CAPACITY);
    }
}

pub(super) fn status_for(result: &CapabilityResult) -> ClientCapabilityStatus {
    match result {
        CapabilityResult::Supported { first_frame_us } => ClientCapabilityStatus::Supported {
            p95_first_frame_us: percentile(first_frame_us, 95),
        },
        CapabilityResult::Unsupported => ClientCapabilityStatus::Unsupported,
        CapabilityResult::Inconclusive => ClientCapabilityStatus::Inconclusive,
    }
}

fn supported_latency(result: &CapabilityResult) -> Option<u64> {
    match status_for(result) {
        ClientCapabilityStatus::Supported { p95_first_frame_us } => Some(p95_first_frame_us),
        _ => None,
    }
}

fn supports(known: &ClientCapabilityProfile, requested: &ClientCapabilityProfile) -> bool {
    let same_codec = known
        .codec()
        .zip(requested.codec())
        .is_some_and(|(known, requested)| known == requested);
    let larger = known
        .dimensions()
        .zip(requested.dimensions())
        .is_some_and(|((kw, kh), (rw, rh))| kw >= rw && kh >= rh);
    same_codec && larger
}

fn percentile(values: &[u64], percent: usize) -> u64 {
    let mut values = values.to_vec();
    values.sort_unstable();
    let rank = percent.saturating_mul(values.len()).div_ceil(100);
    values[rank.saturating_sub(1).min(values.len() - 1)]
}
