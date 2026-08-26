use super::{PartialRangeStore, TransformPublication, TransformPublicationOutcome};
use anyhow::Result;

impl PartialRangeStore {
    /// Publishes a transform without an external authorization callback.
    ///
    /// # Errors
    ///
    /// Returns an error when the publication transaction cannot be committed.
    pub async fn publish_transform(&self, publication: TransformPublication) -> Result<bool> {
        let outcome = self
            .publish_transform_authorized(publication, || true)
            .await?;
        Ok(outcome == TransformPublicationOutcome::Published)
    }
}
