//! Absorbing finished IO: results teach the catalog and the store,
//! failures are charged to the per-source retry policy so the event
//! loop cannot hammer a broken source, and complete files are
//! finalized.

use crate::chunk::downloader::ChunkResult;
use crate::manager::failure::origin_failure_class;
use crate::manager::pressure::is_store_pressure;
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::concurrency::NetworkSetback;
use ghostr_engine::host_stats::host_of;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use ghostr_net::media_log_identity::MediaLogIdentity;
use log::warn;

mod hedge;

impl DeliveryWorker {
    pub(crate) async fn finish_chunk(&mut self, done: ChunkDone) {
        if successful_required_bytes(&done) {
            self.downloads.complete_hedge_winner(done.attempt.id());
        }
        let status = self.downloads.finish(&done.attempt);
        self.observe_chunk_completion(&done, status);
        let identity = done.attempt.identity().clone();
        self.finish_body(&identity);
        match hedge::completion_use(status, &done) {
            hedge::CompletionUse::Useful => {}
            hedge::CompletionUse::OriginEvidence => {
                hedge::record_origin_only(self, &done, &identity);
                return;
            }
            hedge::CompletionUse::Discarded => return,
        }
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

    pub(crate) fn finish_body(&mut self, identity: &TransferIdentity) {
        if !self.downloads.contains_identity(identity) {
            self.probes.body_finished(identity.post());
        }
    }

    /// A transfer that failed on the local store is the device's
    /// problem, not the source's, and must not spend its attempts
    /// (see `delivery_pressure`).
    fn absorb_failure(&mut self, post: &PostId, url: &str, error: &anyhow::Error) {
        if self.absorb_store_pressure(post, error) {
            return;
        }
        let Some(class) = origin_failure_class(error) else {
            return;
        };
        self.note_network_setback(NetworkSetback::Failure);
        warn!(
            "Chunk transfer failed for {} ({class:?})",
            MediaLogIdentity::from_url(url)
        );
        self.note_failed_attempt(post, url, class);
    }

    async fn absorb_chunk(&mut self, identity: &TransferIdentity, result: ChunkResult) {
        if !self
            .learn_transfer(identity, result.total_bytes, result.range_support)
            .await
        {
            return;
        }
        if result.cancelled {
            return;
        }
        let source = identity.source().as_str();
        self.note_successful_attempt(identity.post(), source);
        if result.range_ignored {
            return;
        }
        self.try_finalize(identity, result.total_bytes).await;
    }

    pub(crate) async fn learn_transfer(
        &mut self,
        identity: &TransferIdentity,
        total: Option<u64>,
        ranged: Option<bool>,
    ) -> bool {
        if !self.ctx.store.transfer_is_current(identity).await {
            return false;
        }
        if !self.learn_response_evidence(identity, total, ranged) {
            return false;
        }
        true
    }

    pub(crate) fn learn_response_evidence(
        &mut self,
        identity: &TransferIdentity,
        total: Option<u64>,
        ranged: Option<bool>,
    ) -> bool {
        let facts = LearnedFacts {
            content_length: total,
            accept_ranges: ranged,
            host: host_of(identity.source().as_str()),
        };
        if !self.state.catalog_mut().learn_response_for(identity, facts) {
            return false;
        }
        true
    }

    /// Byte-complete files leave the partial pool whether or not the
    /// note advertised an `imeta x`; an advertised digest still decides
    /// whether the bytes are kept (see `partial_range_completion`).
    /// Bytes that fail that check came from the source, so a failed
    /// finalize is charged to it like any other failed attempt.
    async fn try_finalize(&mut self, identity: &TransferIdentity, total: Option<u64>) {
        let post = identity.post();
        if !self.transfer_complete(post).await {
            return;
        }
        let advertised = self.advertised_digest(post);
        let outcome = self
            .ctx
            .store
            .finalize(post.as_str(), advertised.as_deref())
            .await;
        match outcome {
            Ok(_) => self.learn_finalized(identity, total),
            Err(error) => {
                self.finish_finalize_error(identity, advertised.as_deref(), error)
                    .await
            }
        }
    }

    async fn transfer_complete(&self, post: &PostId) -> bool {
        self.ctx
            .store
            .is_complete(post.as_str())
            .await
            .unwrap_or(false)
    }

    fn advertised_digest(&self, post: &PostId) -> Option<String> {
        self.state
            .catalog()
            .lookup(post)
            .and_then(|entry| entry.meta.sha256.clone())
    }

    fn learn_finalized(&mut self, identity: &TransferIdentity, total: Option<u64>) {
        if let Some(total) = total {
            self.state.catalog_mut().learn_complete_bytes_for(
                identity,
                total,
                crate::manager::time::unix_time_ms(),
            );
        }
    }
}

fn successful_required_bytes(done: &ChunkDone) -> bool {
    done.outcome
        .as_ref()
        .is_ok_and(|result| !result.cancelled && !result.range_ignored && result.bytes_written > 0)
}
