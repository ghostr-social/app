use super::CatalogEntry;
use crate::representation::{RepresentationBinding, RepresentationGeneration, RepresentationId};
use crate::VideoMeta;

impl CatalogEntry {
    pub(super) fn switch(
        &mut self,
        meta: VideoMeta,
        generation: RepresentationGeneration,
        source_representation: Option<RepresentationId>,
    ) {
        self.binding =
            RepresentationBinding::new(self.post.clone(), &meta, generation, source_representation);
        self.meta = meta;
        self.evidence.clear();
        self.ledger = crate::evidence::EvidenceLedger::default();
        self.evidence_clock_ms = 0;
        self.http_clocks.clear();
        self.http_generations.clear();
        self.verified_mirrors.clear();
        self.next_http_generation = 1;
        self.quarantined = false;
        self.timeline = None;
        self.tail_timeline_needed = false;
        self.preview = None;
        self.seed_declared_evidence();
    }
}
