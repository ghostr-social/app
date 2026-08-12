//! Absorbing finished IO: results teach the catalog and the store,
//! failures are charged to the per-source retry policy so the event
//! loop cannot hammer a broken source, and complete files are
//! finalized.

use crate::chunk::downloader::ChunkResult;
use crate::manager::failure::{classify, FailureClass};
use crate::manager::inflight::CompletionStatus;
use crate::manager::pressure::is_store_pressure;
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::concurrency::NetworkSetback;
use ghostr_engine::host_stats::host_of;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use log::warn;

impl DeliveryWorker {
    pub(crate) async fn finish_chunk(&mut self, done: ChunkDone) {
        let status = self.downloads.finish(&done.attempt);
        if !Self::accepts_completion(status) {
            return;
        }
        let identity = done.attempt.identity().clone();
        if !done.outcome.as_ref().err().is_some_and(is_store_pressure) {
            self.keeper.note_chunk(&done);
        }
        // Free space moves while a chunk is in flight — other apps write
        // to the same volume — so every finished chunk re-measures it and
        // evicts down to the cap instead of waiting for the next write.
        self.ctx.store.enforce_capacity().await;
        match done.outcome {
            Ok(result) => self.absorb_chunk(&identity, result).await,
            Err(error) => self.absorb_failure(identity.post(), &done.url, &error),
        }
    }

    fn accepts_completion(status: CompletionStatus) -> bool {
        match status {
            CompletionStatus::Current => true,
            CompletionStatus::Superseded => false,
        }
    }

    /// A transfer that failed on the local store is the device's
    /// problem, not the source's, and must not spend its attempts
    /// (see `delivery_pressure`).
    fn absorb_failure(&mut self, post: &PostId, url: &str, error: &anyhow::Error) {
        if self.absorb_store_pressure(post, error) {
            return;
        }
        self.note_network_setback(NetworkSetback::Failure);
        warn!("Chunk transfer failed: {error:#}");
        self.note_failed_attempt(post, url, classify(error));
    }

    async fn absorb_chunk(&mut self, identity: &TransferIdentity, result: ChunkResult) {
        if !self
            .learn_transfer(identity, result.total_bytes, result.accept_ranges)
            .await
        {
            return;
        }
        if result.bytes_written == 0 && !result.cancelled {
            // The server ignored the range: no progress is possible
            // right now, so it counts as a failed attempt.
            return self.note_failed_attempt(
                identity.post(),
                identity.source().as_str(),
                FailureClass::Transient,
            );
        }
        let source = identity.source().as_str();
        self.note_successful_attempt(identity.post(), source);
        self.try_finalize(identity.post(), source).await;
    }

    pub(crate) async fn learn_transfer(
        &mut self,
        identity: &TransferIdentity,
        total: Option<u64>,
        ranged: bool,
    ) -> bool {
        if !self.ctx.store.transfer_is_current(identity).await {
            return false;
        }
        self.learn_identity(identity, total, ranged).await
    }

    pub(crate) async fn learn_identity(
        &mut self,
        identity: &TransferIdentity,
        total: Option<u64>,
        ranged: bool,
    ) -> bool {
        let facts = LearnedFacts {
            content_length: total,
            accept_ranges: Some(ranged),
            host: host_of(identity.source().as_str()),
        };
        if !self.state.catalog_mut().learn_for(identity, facts) {
            return false;
        }
        if let Some(total) = total {
            self.set_store_total(identity.post(), total).await;
        }
        true
    }

    /// Declares the store total once; a conflicting later fact only
    /// warns — the stored length stays authoritative for serving.
    pub(crate) async fn set_store_total(&self, post: &PostId, total: u64) {
        let key = post.as_str();
        match self.ctx.store.total_len(key).await {
            Ok(Some(_)) => {}
            Ok(None) => self
                .ctx
                .store
                .set_total_len(key, total)
                .await
                .unwrap_or_else(|error| warn!("Total length rejected for {key}: {error:#}")),
            Err(error) => warn!("Store lookup failed for {key}: {error:#}"),
        }
    }

    /// Byte-complete files leave the partial pool whether or not the
    /// note advertised an `imeta x`; an advertised digest still decides
    /// whether the bytes are kept (see `partial_range_completion`).
    /// Bytes that fail that check came from the source, so a failed
    /// finalize is charged to it like any other failed attempt.
    async fn try_finalize(&mut self, post: &PostId, url: &str) {
        if !self
            .ctx
            .store
            .is_complete(post.as_str())
            .await
            .unwrap_or(false)
        {
            return;
        }
        let advertised = self
            .state
            .catalog()
            .lookup(post)
            .and_then(|entry| entry.meta.sha256.clone());
        let outcome = self
            .ctx
            .store
            .finalize(post.as_str(), advertised.as_deref())
            .await;
        if let Err(error) = outcome {
            warn!("Finalize failed for {}: {error:#}", post.as_str());
            self.note_failed_attempt(post, url, FailureClass::Transient);
        }
    }
}
