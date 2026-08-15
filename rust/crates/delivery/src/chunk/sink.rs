use anyhow::Result;
use ghostr_engine::representation::TransferIdentity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::future::Future;

use crate::chunk::generation::OriginGeneration;

/// Explicitly unguarded sink retained for isolated transport tests.
pub struct ChunkSink<'a> {
    pub store: &'a PartialRangeStore,
    pub key: &'a str,
}

pub(crate) struct TransferChunkSink<'a> {
    store: &'a PartialRangeStore,
    identity: TransferIdentity,
}

pub trait ChunkWrite {
    fn accept<'a>(
        &'a self,
        generation: &'a OriginGeneration,
    ) -> impl Future<Output = Result<()>> + Send + 'a;

    fn write<'a>(
        &'a self,
        generation: &'a OriginGeneration,
        offset: u64,
        bytes: &'a [u8],
    ) -> impl Future<Output = Result<bool>> + Send + 'a;
}

impl<'a> TransferChunkSink<'a> {
    pub(crate) fn new(store: &'a PartialRangeStore, identity: TransferIdentity) -> Self {
        Self { store, identity }
    }
}

impl ChunkWrite for ChunkSink<'_> {
    async fn accept<'a>(&'a self, _generation: &'a OriginGeneration) -> Result<()> {
        Ok(())
    }

    async fn write<'a>(
        &'a self,
        _generation: &'a OriginGeneration,
        offset: u64,
        bytes: &'a [u8],
    ) -> Result<bool> {
        self.store.write_range(self.key, offset, bytes).await?;
        Ok(true)
    }
}

impl ChunkWrite for TransferChunkSink<'_> {
    async fn accept<'a>(&'a self, generation: &'a OriginGeneration) -> Result<()> {
        self.store
            .accept_generation(&self.identity, generation.strict()?)
            .await
    }

    async fn write<'a>(
        &'a self,
        generation: &'a OriginGeneration,
        offset: u64,
        bytes: &'a [u8],
    ) -> Result<bool> {
        self.store
            .write_range_for_generation_if_current(
                &self.identity,
                &generation.strict()?,
                offset,
                bytes,
            )
            .await
    }
}
