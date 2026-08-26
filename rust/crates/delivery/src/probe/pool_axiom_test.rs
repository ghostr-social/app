use super::*;

impl MetadataProbePool {
    /// Window posts with unresolved size or range support, bounded by the limit.
    /// Returned posts are marked as probing until released or learned.
    /// Sources the retry policy retired are never probed again.
    pub(crate) fn claim(
        &mut self,
        catalog: &Catalog,
        window: &[PostId],
        retry: &RetryBook,
    ) -> Vec<(PostId, String)> {
        let mut claimed = Vec::new();
        for post in window {
            if self.probing.len() >= self.limit {
                break;
            }
            if let Some(url) = self.needed_probe(catalog, retry, post) {
                let identity = catalog
                    .transfer_identity(post, &url)
                    .expect("probe source came from the catalog");
                self.probing
                    .insert(post.clone(), ActiveProbe::new(identity));
                claimed.push((post.clone(), url));
            }
        }
        claimed
    }
    pub(crate) fn learned(
        &mut self,
        identity: &TransferIdentity,
        generation: Option<ghostr_engine::representation::HttpGenerationLease>,
    ) {
        self.probing.remove(identity.post());
        self.deferred.remove(identity.post());
        self.probed
            .insert(identity.clone(), CompletedHeadProbe::for_test(generation));
    }
    pub(crate) fn has_completed_identity(&self, identity: &TransferIdentity) -> bool {
        self.probed.contains_key(identity)
    }
}
