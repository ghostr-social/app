use super::{ActiveChunk, InFlightChunks};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ActionId, ByteRange, PostId};
use std::collections::{HashMap, HashSet};

impl InFlightChunks {
    pub(crate) fn cancel_action(&mut self, action: ActionId) -> bool {
        let Some(active) = self.transfers.get_mut(&action) else {
            return false;
        };
        active.cancel()
    }

    pub(crate) fn can_cancel_action(&self, action: ActionId) -> bool {
        self.transfers
            .get(&action)
            .is_some_and(|active| !active.cancelling && !active.io_finished())
    }

    pub(crate) fn cancel_obsolete(&mut self, binding: &RepresentationBinding) {
        for active in self.transfers.values_mut() {
            let current = binding.transfer(active.identity.source().as_str());
            let obsolete =
                active.chunk.post == *binding.post() && current.as_ref() != Some(&active.identity);
            if obsolete {
                active.cancel();
            }
        }
    }

    pub(crate) fn cancel_without_body(&mut self, posts: &HashSet<PostId>) {
        self.transfers
            .values_mut()
            .filter(|active| posts.contains(&active.chunk.post))
            .filter(|active| !active.body_received())
            .for_each(|active| {
                active.cancel();
            });
    }

    pub(crate) fn cancel_covered_without_body(
        &mut self,
        present: &HashMap<PostId, Vec<ByteRange>>,
        transformed: &HashMap<PostId, RepresentationBinding>,
        catalog: &Catalog,
    ) {
        self.transfers
            .values_mut()
            .filter(|active| covered_without_body(active, present, transformed, catalog))
            .for_each(|active| {
                active.cancel();
            });
    }
}

fn covered_without_body(
    active: &ActiveChunk,
    present: &HashMap<PostId, Vec<ByteRange>>,
    transformed: &HashMap<PostId, RepresentationBinding>,
    catalog: &Catalog,
) -> bool {
    cancellable_before_body(active)
        && !transformed.contains_key(&active.chunk.post)
        && current_transfer(active, catalog)
        && present
            .get(&active.chunk.post)
            .is_some_and(|ranges| covers(ranges, active.effective_bytes))
}

fn cancellable_before_body(active: &ActiveChunk) -> bool {
    !active.cancelling
        && !active.io_finished()
        && !active.body_received()
        && !active.effective_bytes.is_empty()
}

fn current_transfer(active: &ActiveChunk, catalog: &Catalog) -> bool {
    catalog
        .binding(&active.chunk.post)
        .and_then(|binding| binding.transfer(active.identity.source().as_str()))
        .is_some_and(|identity: TransferIdentity| identity == active.identity)
}

fn covers(present: &[ByteRange], target: ByteRange) -> bool {
    ghostr_engine::media_timeline::normalize(present.to_vec())
        .iter()
        .any(|range| range.start <= target.start && range.end >= target.end)
}

#[cfg(test)]
#[path = "cancellation_axiom_test.rs"]
pub(crate) mod axiom_test_support;
