use super::AdaptiveRecord;
use std::collections::BTreeMap;

pub(super) fn retain_oldest<K: Clone + Ord>(records: &mut BTreeMap<K, AdaptiveRecord>, cap: usize) {
    while records.len() > cap {
        let victim = records
            .iter()
            .min_by_key(|(key, record)| (record.last_at_ms(), *key))
            .map(|(key, _)| key.clone());
        let Some(victim) = victim else { return };
        records.remove(&victim);
    }
}
