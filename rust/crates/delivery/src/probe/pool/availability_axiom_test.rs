use super::*;

use crate::manager::retry::RetryBook;

use ghostr_engine::catalog::Catalog;

use ghostr_engine::PostId;

impl MetadataProbePool {
    pub(in super::super) fn needed_probe(
        &self,
        catalog: &Catalog,
        retry: &RetryBook,
        post: &PostId,
    ) -> Option<String> {
        if self.probing.contains_key(post) || retry.is_cooling(post) {
            return None;
        }
        let entry = catalog.lookup(post)?;
        let url = retry.live_urls(post, &entry.meta.urls).into_iter().next()?;
        let identity = catalog.transfer_identity(post, &url)?;
        if self.deferred.contains(&identity) {
            return None;
        }
        if self
            .probed
            .get(&identity)
            .is_some_and(|history| history.current(catalog, &identity))
        {
            return None;
        }
        if evidence_complete(&entry.evidence_assessment_for(&url, 0)) {
            return None;
        }
        Some(url)
    }
}
