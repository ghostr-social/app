use super::{PartialRangeStore, ResponseOwnerRef, Result, TransferIdentity};

impl PartialRangeStore {
    /// Finishes a response owned by a legacy test action.
    ///
    /// # Errors
    ///
    /// Returns an error when validation, rollback, or publication fails.
    pub async fn finish_single_response(
        &self,
        identity: &TransferIdentity,
        action: u64,
        total: Option<u64>,
        complete: bool,
    ) -> Result<bool> {
        self.finish_single_response_owned(
            identity,
            ResponseOwnerRef::Legacy(action),
            total,
            complete,
        )
        .await
    }
}
