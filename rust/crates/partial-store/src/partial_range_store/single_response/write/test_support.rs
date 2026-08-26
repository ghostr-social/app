use super::{PartialRangeStore, ResponseOwnerRef, Result, SingleResponseWrite, TransferIdentity};

impl PartialRangeStore {
    /// Writes bytes while a legacy response action remains authoritative.
    ///
    /// # Errors
    ///
    /// Returns an error when the response cannot validate or persist the write.
    pub async fn write_single_response_if_current(
        &self,
        identity: &TransferIdentity,
        action: u64,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        let write = SingleResponseWrite {
            owner: ResponseOwnerRef::Legacy(action),
            reservation: None,
            offset,
            bytes,
        };
        self.write_single_response(identity, write).await
    }
}
