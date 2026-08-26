use super::{ContentRevision, PartialRangeStore, RepresentationBinding};

impl PartialRangeStore {
    pub(crate) async fn stream_snapshot(
        &self,
        key: &str,
    ) -> (Option<RepresentationBinding>, ContentRevision) {
        let Ok(_update) = self.observe_key(key).await else {
            return (None, self.current_content_revision(key).await);
        };
        let binding = self.representations.lock().await.get(key).cloned();
        let revision = self.current_content_revision(key).await;
        (binding, revision)
    }
}
