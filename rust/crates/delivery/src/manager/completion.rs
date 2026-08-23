//! Absorbing finished IO: results teach the catalog and the store,
//! failures are charged to the per-source retry policy so the event
//! loop cannot hammer a broken source, and complete files are
//! finalized.

use crate::chunk::downloader::ChunkResult;
use crate::manager::failure::origin_failure_class;
use crate::manager::pressure::is_store_pressure;
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use ghostr_engine::representation::{HttpGenerationLease, TransferIdentity};
use ghostr_engine::PostId;
use ghostr_net::media_log_identity::MediaLogIdentity;
use log::warn;

mod evidence;
mod finalize;
mod hedge;
mod policy_limit;

impl DeliveryWorker {
    pub(crate) async fn finish_chunk(&mut self, done: ChunkDone) {
        self.record_whole_body_limit(&done);
        let generation = self.downloads.http_generation(&done.attempt);
        let whole_body_completed = self.learn_network_completion(&done, generation.as_ref());
        if successful_required_bytes(&done) {
            self.downloads.complete_hedge_winner(done.attempt.id());
        }
        let finished = self.downloads.finish(&done.attempt);
        let status = finished.status();
        self.observe_chunk_completion(&done, finished);
        let identity = done.attempt.identity().clone();
        self.finish_body(&identity);
        if !self.retain_completion(status, &done, &identity) {
            return;
        }
        self.finish_useful_chunk(done, &identity, whole_body_completed, generation)
            .await;
    }

    fn retain_completion(
        &mut self,
        status: crate::manager::inflight::CompletionStatus,
        done: &ChunkDone,
        identity: &TransferIdentity,
    ) -> bool {
        match hedge::completion_use(status, &done) {
            hedge::CompletionUse::Useful => true,
            hedge::CompletionUse::OriginEvidence => {
                hedge::record_origin_only(self, &done, &identity);
                false
            }
            hedge::CompletionUse::Discarded => false,
        }
    }

    async fn finish_useful_chunk(
        &mut self,
        done: ChunkDone,
        identity: &TransferIdentity,
        whole_body_completed: bool,
        generation: Option<HttpGenerationLease>,
    ) {
        if !done.outcome.as_ref().err().is_some_and(is_store_pressure) {
            self.keeper.note_chunk(&done);
        }
        // Free space moves while a chunk is in flight — other apps write
        // to the same volume — so every finished chunk re-measures it and
        // evicts down to the cap instead of waiting for the next write.
        self.ctx.store.enforce_capacity().await;
        let response = done.response_evidence.clone();
        match done.outcome {
            Ok(result) => {
                self.absorb_chunk(
                    identity,
                    result,
                    AbsorbEvidence {
                        size_already_learned: whole_body_completed,
                        response: response.as_ref(),
                        generation: generation.as_ref(),
                    },
                )
                .await
            }
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
        if crate::chunk::sink::is_local_store_failure(error) {
            warn!("Local media publication failed; pausing before retry");
            self.start_cooldown(post.clone(), self.pressure.retry_delay());
            return;
        }
        let Some(class) = origin_failure_class(error) else {
            return;
        };
        warn!(
            "Chunk transfer failed for {} ({class:?})",
            MediaLogIdentity::from_url(url)
        );
        self.note_failed_attempt(post, url, class);
    }

    async fn absorb_chunk(
        &mut self,
        identity: &TransferIdentity,
        result: ChunkResult,
        evidence: AbsorbEvidence<'_>,
    ) {
        if !self.transfer_is_current(identity).await {
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
        let total = (!evidence.size_already_learned)
            .then_some(result.total_bytes)
            .flatten();
        self.try_finalize(identity, total, evidence.response, evidence.generation)
            .await;
    }
}

struct AbsorbEvidence<'a> {
    size_already_learned: bool,
    response: Option<&'a crate::chunk::downloader::HttpResponseEvidence>,
    generation: Option<&'a HttpGenerationLease>,
}

fn successful_required_bytes(done: &ChunkDone) -> bool {
    done.outcome
        .as_ref()
        .is_ok_and(|result| !result.cancelled && !result.range_ignored && result.bytes_written > 0)
}
