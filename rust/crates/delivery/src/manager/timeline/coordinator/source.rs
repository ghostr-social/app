use super::TimelineCoordinator;
use crate::chunk::downloader::OpenedResponse;
use crate::manager::timeline::TimelineEvidence;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use ghostr_net::media_retention::MediaRetention;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;

impl TimelineCoordinator {
    pub(crate) fn observe_index_source(
        &mut self,
        identity: &TransferIdentity,
        response: &OpenedResponse,
    ) {
        let source = response
            .generation()
            .filter(|_| response.retention() == MediaRetention::Public);
        self.sources.remove(identity.post());
        if let Some(source) = source {
            self.sources
                .insert(identity.post().clone(), (identity.clone(), source.clone()));
        }
    }

    pub(crate) fn evidence(
        &self,
        binding: &RepresentationBinding,
        snapshot: &StoredMediaSnapshot,
    ) -> Option<TimelineEvidence> {
        let mut evidence = TimelineEvidence::from_snapshot(binding, snapshot)?;
        evidence.source = self
            .sources
            .get(binding.post())
            .filter(|(identity, source)| {
                binding.transfer(identity.source().as_str()).as_ref() == Some(identity)
                    && source.total_bytes() == evidence.total()
            })
            .cloned();
        Some(evidence)
    }
}
