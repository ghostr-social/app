use super::{Confidence, Evidence, EvidenceField, EvidenceScope, EvidenceValue};

const NETWORK_HALF_LIFE_MS: u64 = 6 * 60 * 60 * 1_000;
const STALE_BPS: u16 = 1_000;

pub(in crate::evidence) fn effective_confidence(
    item: &Evidence<EvidenceValue>,
    now_ms: u64,
) -> Confidence {
    let stable = item.observed_at_ms == 0
        || item.source.structural() && matches!(item.scope, EvidenceScope::ImmutableBytes(_))
        || (item.value.field() != EvidenceField::RangeSupport
            && item
                .validator
                .as_ref()
                .is_some_and(|value| value.is_strong()));
    match stable {
        true => item.confidence,
        false => item.confidence.decayed(
            now_ms.saturating_sub(item.observed_at_ms),
            NETWORK_HALF_LIFE_MS,
        ),
    }
}

pub(in crate::evidence) fn fresh<'a>(
    records: &[&'a Evidence<EvidenceValue>],
    now_ms: u64,
) -> Vec<&'a Evidence<EvidenceValue>> {
    records
        .iter()
        .copied()
        .filter(|item| is_fresh(item, now_ms))
        .collect()
}

pub(in crate::evidence) fn is_fresh(item: &Evidence<EvidenceValue>, now_ms: u64) -> bool {
    effective_confidence(item, now_ms).basis_points() >= STALE_BPS
}
