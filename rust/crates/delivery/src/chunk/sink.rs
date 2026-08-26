use anyhow::Result;
use core::fmt::{Display, Formatter};
use core::future::Future;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::TransferIdentity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use ghostr_partial_store::partial_range_store::StoreAction;

use crate::chunk::generation::OriginGeneration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseWriteMode {
    Sparse,
    SingleResponse(WholeBodyContract),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LocalStoreFailure;

impl Display for LocalStoreFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("local media store operation failed")
    }
}

impl core::error::Error for LocalStoreFailure {}

pub(crate) fn is_local_store_failure(error: &anyhow::Error) -> bool {
    error.is::<LocalStoreFailure>()
}

/// Explicitly unguarded sink retained for isolated transport tests.
pub struct ChunkSink<'a> {
    pub store: &'a PartialRangeStore,
    pub key: &'a str,
}

pub(crate) struct TransferChunkSink<'a> {
    store: &'a PartialRangeStore,
    identity: TransferIdentity,
    action: StoreAction,
}

pub trait ChunkWrite {
    fn accept<'a>(
        &'a self,
        generation: &'a OriginGeneration,
        mode: ResponseWriteMode,
    ) -> impl Future<Output = Result<()>> + Send + 'a;

    fn write<'a>(
        &'a self,
        generation: &'a OriginGeneration,
        mode: ResponseWriteMode,
        offset: u64,
        bytes: &'a [u8],
    ) -> impl Future<Output = Result<bool>> + Send + 'a;

    fn finish<'a>(
        &'a self,
        generation: &'a OriginGeneration,
        mode: ResponseWriteMode,
        total: Option<u64>,
        complete: bool,
    ) -> impl Future<Output = Result<bool>> + Send + 'a;
}

impl<'a> TransferChunkSink<'a> {
    pub(crate) fn new(
        store: &'a PartialRangeStore,
        identity: TransferIdentity,
        action: StoreAction,
    ) -> Self {
        Self {
            store,
            identity,
            action,
        }
    }
}

impl ChunkWrite for ChunkSink<'_> {
    fn accept<'a>(
        &'a self,
        _generation: &'a OriginGeneration,
        _mode: ResponseWriteMode,
    ) -> impl Future<Output = Result<()>> + Send + 'a {
        core::future::ready(Ok(()))
    }

    async fn write<'a>(
        &'a self,
        _generation: &'a OriginGeneration,
        _mode: ResponseWriteMode,
        offset: u64,
        bytes: &'a [u8],
    ) -> Result<bool> {
        self.store.write_range(self.key, offset, bytes).await?;
        Ok(true)
    }

    async fn finish<'a>(
        &'a self,
        _generation: &'a OriginGeneration,
        _mode: ResponseWriteMode,
        total: Option<u64>,
        complete: bool,
    ) -> Result<bool> {
        if complete {
            self.store
                .set_total_len(self.key, total.unwrap_or_default())
                .await?;
        }
        Ok(true)
    }
}

impl ChunkWrite for TransferChunkSink<'_> {
    fn accept<'a>(
        &'a self,
        _generation: &'a OriginGeneration,
        _mode: ResponseWriteMode,
    ) -> impl Future<Output = Result<()>> + Send + 'a {
        core::future::ready(Ok(()))
    }

    async fn write<'a>(
        &'a self,
        generation: &'a OriginGeneration,
        mode: ResponseWriteMode,
        offset: u64,
        bytes: &'a [u8],
    ) -> Result<bool> {
        if mode == ResponseWriteMode::Sparse && generation.is_resumable() {
            return self
                .store
                .write_range_for_action_if_current(
                    &self.identity,
                    &generation.strict()?,
                    &self.action,
                    offset,
                    bytes,
                )
                .await;
        }
        self.store
            .write_single_response_for_action(&self.identity, &self.action, offset, bytes)
            .await
    }

    async fn finish<'a>(
        &'a self,
        generation: &'a OriginGeneration,
        mode: ResponseWriteMode,
        total: Option<u64>,
        complete: bool,
    ) -> Result<bool> {
        if mode == ResponseWriteMode::Sparse && generation.is_resumable() {
            return self
                .store
                .finish_sparse_response(&self.identity, &generation.strict()?, &self.action)
                .await;
        }
        self.store
            .finish_single_response_for_action(&self.identity, &self.action, total, complete)
            .await
    }
}
