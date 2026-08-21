use super::MetadataProbePool;
use crate::manager::retry::RetryBook;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::PostId;

impl MetadataProbePool {
    #[cfg(test)]
    pub(super) fn needed_probe(
        &self,
        catalog: &Catalog,
        retry: &RetryBook,
        post: &PostId,
    ) -> Option<String> {
        if self.probing.contains_key(post)
            || self.probed.contains(post)
            || self.deferred.contains(post)
            || retry.is_cooling(post)
        {
            return None;
        }
        let entry = catalog.lookup(post)?;
        let url = retry.live_urls(post, &entry.meta.urls).into_iter().next()?;
        if entry.planning_total_for(&url).is_some()
            && entry.observed_range_support_for(&url).is_some()
        {
            return None;
        }
        Some(url)
    }

    pub(super) fn can_probe(
        &self,
        catalog: &Catalog,
        retry: &RetryBook,
        post: &PostId,
        source: &str,
    ) -> bool {
        if self.probing.contains_key(post)
            || self.probed.contains(post)
            || self.deferred.contains(post)
            || retry.is_cooling(post)
        {
            return false;
        }
        let Some(entry) = catalog.lookup(post) else {
            return false;
        };
        retry
            .live_urls(post, &entry.meta.urls)
            .iter()
            .any(|url| url == source)
            && (entry.planning_total_for(source).is_none()
                || entry.observed_range_support_for(source).is_none())
    }
}
