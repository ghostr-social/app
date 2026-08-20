use super::{
    CacheState, CachedHlsObject, FocusRecord, SegmentedPhase, SegmentedSnapshot, MAX_CACHE_BYTES,
};
use crate::segmented::PreparedHls;

pub(super) struct CacheCommit {
    pub snapshot: SegmentedSnapshot,
    pub objects: Vec<String>,
}

pub(super) fn commit(state: &mut CacheState, prepared: PreparedHls) -> CacheCommit {
    let bytes_present = prepared.bytes_present();
    let objects = prepared
        .objects
        .iter()
        .map(|object| object.request_url.clone())
        .collect();
    for object in prepared.objects {
        let cached = CachedHlsObject::new(object.body, object.final_url, object.content_type);
        insert(state, object.request_url, cached);
    }
    CacheCommit {
        snapshot: SegmentedSnapshot {
            phase: SegmentedPhase::Ready,
            bytes_present,
            eta_ms: Some(0),
            detail: None,
        },
        objects,
    }
}

pub(super) fn failed(detail: String) -> CacheCommit {
    CacheCommit {
        snapshot: SegmentedSnapshot {
            phase: SegmentedPhase::Failed,
            bytes_present: 0,
            eta_ms: None,
            detail: Some(detail),
        },
        objects: Vec::new(),
    }
}

fn insert(state: &mut CacheState, key: String, object: CachedHlsObject) {
    if let Some(replaced) = state.objects.remove(&key) {
        state.bytes = state.bytes.saturating_sub(replaced.body.len());
    }
    make_room(state, object.body.len());
    state.aliases.retain(|_, canonical| canonical != &key);
    state
        .aliases
        .insert(object.final_url.to_string(), key.clone());
    state.bytes = state.bytes.saturating_add(object.body.len());
    state.order.retain(|known| known != &key);
    state.order.push_back(key.clone());
    state.objects.insert(key, object);
}

fn make_room(state: &mut CacheState, incoming: usize) {
    while state.bytes.saturating_add(incoming) > MAX_CACHE_BYTES {
        let Some(oldest) = state.order.pop_front() else {
            break;
        };
        if let Some(removed) = state.objects.remove(&oldest) {
            state.bytes = state.bytes.saturating_sub(removed.body.len());
            state.aliases.retain(|_, canonical| canonical != &oldest);
            invalidate(state.focus.values_mut(), &oldest);
        }
    }
}

fn invalidate<'a>(records: impl Iterator<Item = &'a mut FocusRecord>, evicted: &str) {
    for record in records {
        if record.objects.iter().any(|key| key == evicted) {
            record.snapshot = SegmentedSnapshot::default();
            record.objects.clear();
        }
    }
}
