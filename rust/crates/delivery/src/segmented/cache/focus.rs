use super::{objects, FocusRecord, SegmentedCache, SegmentedPhase, SegmentedSnapshot};
use ghostr_engine::representation::RepresentationId;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};

pub(crate) type PreservedFocus = HashMap<PostId, (u64, String)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SegmentedFocusItem {
    post: PostId,
    representation_id: RepresentationId,
    sources: Vec<String>,
}

impl SegmentedFocusItem {
    pub(crate) fn new(
        post: PostId,
        representation_id: RepresentationId,
        sources: Vec<String>,
    ) -> Self {
        Self {
            post,
            representation_id,
            sources,
        }
    }

    pub(crate) fn post(&self) -> &PostId {
        &self.post
    }

    fn representation_id(&self) -> &RepresentationId {
        &self.representation_id
    }

    pub(crate) fn sources(&self) -> &[String] {
        &self.sources
    }
}

impl SegmentedCache {
    pub(crate) fn reconcile_focus_window(
        &self,
        generation: u64,
        items: Vec<SegmentedFocusItem>,
        protected: &HashSet<PostId>,
        preserved: &PreservedFocus,
    ) {
        let mut state = self.lock();
        let mut next = HashMap::new();
        for item in items {
            let previous = state.focus.remove(item.post());
            let mut record = reconcile_record(
                previous,
                RecordInput {
                    generation,
                    item: &item,
                    protected,
                    preserved,
                },
            );
            rebind_ready_authority(&mut state, &mut record, &item);
            next.insert(item.post, record);
        }
        state.focus = next;
        objects::retain_referenced(&mut state);
        drop(state);
        self.changed.notify_waiters();
    }

    pub(crate) fn root_source(&self, post: &PostId) -> Option<String> {
        self.lock().focus.get(post)?.root_source.clone()
    }

    pub(crate) fn focus_generation(&self, post: &PostId) -> Option<u64> {
        self.lock().focus.get(post).map(|record| record.generation)
    }
}

struct RecordInput<'a> {
    generation: u64,
    item: &'a SegmentedFocusItem,
    protected: &'a HashSet<PostId>,
    preserved: &'a PreservedFocus,
}

fn reconcile_record(previous: Option<FocusRecord>, input: RecordInput<'_>) -> FocusRecord {
    let active = input.preserved.get(input.item.post());
    let reusable = previous.filter(|record| reusable(record, input.item.sources(), active));
    let mut record = reusable.unwrap_or_else(|| empty_record(input.generation, input.item));
    record.sources = input.item.sources.clone();
    record.protected = input.protected.contains(input.item.post());
    record
}

fn rebind_ready_authority(
    state: &mut super::CacheState,
    record: &mut FocusRecord,
    item: &SegmentedFocusItem,
) {
    if record.representation_id == *item.representation_id() {
        return;
    }
    record.representation_id = item.representation_id.clone();
    if record.snapshot.phase != SegmentedPhase::Ready {
        return;
    }
    let revision = super::SegmentedAssetRevision::allocate(&mut state.last_asset_revision);
    record.snapshot.authority = Some(super::HlsPreparedAssetAuthority::new(
        item.post.clone(),
        item.representation_id.clone(),
        revision,
    ));
}

fn reusable(record: &FocusRecord, sources: &[String], active: Option<&(u64, String)>) -> bool {
    let ready = record.snapshot.phase == SegmentedPhase::Ready
        && record
            .root_source
            .as_deref()
            .is_some_and(|root| super::super::source_key::contains(sources, root));
    let working = active.is_some_and(|(generation, root)| {
        record.generation == *generation && super::super::source_key::contains(sources, root)
    });
    ready || working
}

fn empty_record(generation: u64, item: &SegmentedFocusItem) -> FocusRecord {
    FocusRecord {
        generation,
        representation_id: item.representation_id.clone(),
        sources: Vec::new(),
        root_source: None,
        protected: false,
        snapshot: SegmentedSnapshot::default(),
        objects: Vec::new(),
        staged: Vec::new(),
        preparing: None,
        reserved_bytes: 0,
        assembly_bytes: 0,
    }
}

#[cfg(test)]
#[path = "focus_axiom_test.rs"]
pub(crate) mod axiom_test_support;
