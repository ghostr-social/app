use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::ChunkId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveRange {
    chunk: ChunkId,
    identity: TransferIdentity,
    committed_until_ms: u64,
}

impl ActiveRange {
    pub(crate) fn new(chunk: ChunkId, identity: TransferIdentity, committed_until_ms: u64) -> Self {
        Self {
            chunk,
            identity,
            committed_until_ms,
        }
    }

    pub(crate) fn chunk(&self) -> &ChunkId {
        &self.chunk
    }

    pub(crate) fn identity(&self) -> &TransferIdentity {
        &self.identity
    }

    pub(crate) fn committed_until_ms(&self) -> u64 {
        self.committed_until_ms
    }
}
