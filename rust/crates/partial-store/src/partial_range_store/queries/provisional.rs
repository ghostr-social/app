use super::PartialRangeStore;
use crate::partial_range_manifest::RangeManifest;
use anyhow::Result;
use sha2::Digest;
use std::ops::Range;

pub(super) struct ProvisionalInterval {
    span: Range<u64>,
    sha256: String,
}

pub(super) async fn capture(store: &PartialRangeStore, key: &str) -> Vec<ProvisionalInterval> {
    store
        .sparse_response_actions
        .lock()
        .await
        .values()
        .filter(|state| readable(state, key))
        .map(|state| ProvisionalInterval {
            span: state.range.start..state.next_offset,
            sha256: format!("{:x}", state.hasher.clone().finalize()),
        })
        .collect()
}

pub(super) fn merge(
    stable: &RangeManifest,
    provisional: &[ProvisionalInterval],
) -> Result<RangeManifest> {
    let mut readable = stable.clone();
    for interval in provisional {
        readable.insert(interval.span.clone())?;
        readable.record_checksum(interval.span.clone(), interval.sha256.clone())?;
    }
    Ok(readable)
}

fn readable(state: &&super::super::generation::SparseResponseState, key: &str) -> bool {
    state.identity.post().as_str() == key
        && state.owner.is_active()
        && state.received > 0
        && state.received == state.next_offset - state.range.start
        && !state.dirty
        && !state.committed
}
