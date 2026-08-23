use super::{InflightKey, StageLease};
use crate::segmented::cache::blocks::commit::{commit_partial, commit_prepared};
use crate::segmented::prepare::{PreparedComplete, PreparedObject};

impl StageLease {
    pub(in crate::segmented) fn commit_partial(mut self, object: PreparedObject) -> bool {
        let Some(key) = self.key.as_ref() else {
            return false;
        };
        if !self.cache.commit_leased_partial(key, object) {
            return false;
        }
        self.key = None;
        true
    }

    pub(in crate::segmented) fn commit_complete(mut self, object: PreparedComplete) -> bool {
        let Some(key) = self.key.as_ref() else {
            return false;
        };
        if !self.cache.commit_leased_complete(key, object) {
            return false;
        }
        self.key = None;
        true
    }
}

impl crate::segmented::SegmentedCache {
    fn commit_leased_partial(&self, key: &InflightKey, object: PreparedObject) -> bool {
        let mut state = self.lock();
        if !accepts_partial(&state, key, &object) {
            return false;
        }
        let record = state.focus.get_mut(&key.post).expect("validated HLS focus");
        if commit_partial(record, key.fence.request.offset, object).is_none() {
            return false;
        }
        record.preparing = None;
        state.inflight.remove(key);
        drop(state);
        self.changed.notify_waiters();
        true
    }

    fn commit_leased_complete(&self, key: &InflightKey, object: PreparedComplete) -> bool {
        let mut state = self.lock();
        if !accepts_complete(&state, key, &object) {
            return false;
        }
        state.inflight.remove(key).expect("validated HLS lease");
        let record = state.focus.get_mut(&key.post).expect("validated HLS focus");
        commit_prepared(record, object);
        record.preparing = None;
        drop(state);
        self.changed.notify_waiters();
        true
    }
}

fn accepts_partial(
    state: &crate::segmented::cache::CacheState,
    key: &InflightKey,
    object: &PreparedObject,
) -> bool {
    let Some((record, inflight)) = state.focus.get(&key.post).zip(state.inflight.get(key)) else {
        return false;
    };
    if record.preparing.as_ref() != Some(&key.fence) || inflight.prefix.is_some() {
        return false;
    }
    if !partial_object_matches(key, object) {
        return false;
    }
    accepts_partial_offset(record, key, object)
}

fn accepts_complete(
    state: &crate::segmented::cache::CacheState,
    key: &InflightKey,
    prepared: &PreparedComplete,
) -> bool {
    let Some((record, inflight)) = state.focus.get(&key.post).zip(state.inflight.get(key)) else {
        return false;
    };
    if !complete_owner_matches(record, key, prepared) {
        return false;
    }
    match &inflight.prefix {
        None => complete_without_prefix(key, prepared),
        Some((_, prefix)) => complete_with_prefix(prefix, key, prepared),
    }
}

fn partial_object_matches(key: &InflightKey, object: &PreparedObject) -> bool {
    (
        object.request_url.as_str(),
        object.body.is_empty(),
        object.body.len() as u64 <= key.fence.request.block_bytes,
    ) == (key.fence.request.url.as_str(), false, true)
}

fn accepts_partial_offset(
    record: &crate::segmented::cache::FocusRecord,
    key: &InflightKey,
    object: &PreparedObject,
) -> bool {
    if key.fence.request.offset == 0 {
        return true;
    }
    record
        .staged
        .iter()
        .any(|known| known.matches_identity(object, key.fence.request.offset))
}

fn complete_owner_matches(
    record: &crate::segmented::cache::FocusRecord,
    key: &InflightKey,
    prepared: &PreparedComplete,
) -> bool {
    (
        record.preparing.as_ref(),
        prepared.object.request_url.as_str(),
    ) == (Some(&key.fence), key.fence.request.url.as_str())
}

fn complete_without_prefix(key: &InflightKey, prepared: &PreparedComplete) -> bool {
    (
        key.fence.request.offset,
        prepared.object.body.is_empty(),
        prepared.object.body.len() as u64 <= key.fence.request.block_bytes,
    ) == (0, false, true)
}

fn complete_with_prefix(
    prefix: &crate::segmented::cache::StagedObject,
    key: &InflightKey,
    prepared: &PreparedComplete,
) -> bool {
    let total = key
        .fence
        .request
        .offset
        .checked_add(key.fence.request.block_bytes);
    (
        prefix.matches_identity(&prepared.object, key.fence.request.offset),
        Some(prepared.object.body.len() as u64),
    ) == (true, total)
}
