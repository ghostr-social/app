//! Choosing what to give back when the cap moves under the store:
//! least recently used first, never a leased video, never the video the
//! caller is writing right now.

use crate::partial_range_disk::Entry;
use crate::partial_range_store::Entries;
use std::collections::BTreeMap;

struct Candidate<'a> {
    touched: u64,
    key: &'a str,
    bytes: u64,
}

/// Keys to discard, oldest use first, until `wanted` bytes are covered.
/// Protected and leased keys are skipped even when that leaves the
/// caller short — refusing a write is better than breaking playback.
pub(super) fn victims(
    entries: &Entries,
    staged: &BTreeMap<String, u64>,
    wanted: u64,
    protected: &str,
    leased: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut candidates: Vec<Candidate<'_>> = entries
        .iter()
        .filter(|(key, _)| key.as_str() != protected && !leased(key))
        .map(|(key, entry)| candidate(key, entry, staged.get(key).copied().unwrap_or_default()))
        .collect();
    candidates.sort_by_key(|candidate| (candidate.touched, candidate.key));
    take_until(candidates, wanted)
}

fn candidate<'a>(key: &'a str, entry: &Entry, staged: u64) -> Candidate<'a> {
    Candidate {
        touched: entry.touched,
        key,
        bytes: entry.accounted.saturating_add(staged),
    }
}

fn take_until(candidates: Vec<Candidate<'_>>, wanted: u64) -> Vec<String> {
    let mut freed = 0_u64;
    let mut chosen = Vec::new();
    for candidate in candidates {
        if freed >= wanted {
            break;
        }
        freed = freed.saturating_add(candidate.bytes);
        chosen.push(candidate.key.to_owned());
    }
    chosen
}
