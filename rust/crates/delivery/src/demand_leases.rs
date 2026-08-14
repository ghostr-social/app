use crate::playback_demand::{ConsumerId, DemandLease, DemandState};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct DemandLeases {
    active: HashMap<ConsumerId, DemandLease>,
}

impl DemandLeases {
    pub(crate) fn apply(&mut self, state: DemandState) {
        match state {
            DemandState::Blocked(lease) | DemandState::Advanced(lease) => {
                self.active.insert(lease.consumer(), lease);
            }
            DemandState::Released(consumer) => {
                self.active.remove(&consumer);
            }
        }
    }

    pub(crate) fn reconcile(
        &mut self,
        foreground: &HashSet<PostId>,
        catalog: &Catalog,
        present: &HashMap<PostId, Vec<ByteRange>>,
    ) -> HashMap<PostId, ByteRange> {
        self.active.retain(|_, lease| valid(lease, catalog));
        active_ranges(self.active.values(), foreground, present)
    }

    pub(crate) fn clear(&mut self) {
        self.active.clear();
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.active.len()
    }
}

fn valid(lease: &DemandLease, catalog: &Catalog) -> bool {
    catalog.lookup(lease.post()).is_some()
        && catalog.binding(lease.post()).as_ref() == lease.representation()
}

fn active_ranges<'a>(
    leases: impl Iterator<Item = &'a DemandLease>,
    foreground: &HashSet<PostId>,
    present: &HashMap<PostId, Vec<ByteRange>>,
) -> HashMap<PostId, ByteRange> {
    let mut ranges = HashMap::new();
    let active = leases.filter(|lease| {
        foreground.contains(lease.post()) && !covered(lease.range(), present.get(lease.post()))
    });
    for lease in active {
        ranges
            .entry(lease.post().clone())
            .and_modify(|active: &mut ByteRange| *active = earlier(*active, lease.range()))
            .or_insert(lease.range());
    }
    ranges
}

fn earlier(left: ByteRange, right: ByteRange) -> ByteRange {
    match (left.start, left.end) <= (right.start, right.end) {
        true => left,
        false => right,
    }
}

fn covered(range: ByteRange, have: Option<&Vec<ByteRange>>) -> bool {
    have.is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.start <= range.start && span.end >= range.end)
    })
}
