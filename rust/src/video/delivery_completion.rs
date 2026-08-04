//! Absorbing finished IO: results teach the catalog and the store,
//! failures start a per-post cooldown so the event loop cannot hammer
//! a broken host, and complete verified files are finalized.

use crate::engine::catalog::LearnedFacts;
use crate::engine::host_stats::host_of;
use crate::engine::PostId;
use crate::video::chunk_downloader::ChunkResult;
use crate::video::delivery_manager::DeliveryWorker;
use crate::video::delivery_transfers::{ChunkDone, InternalEvent, ProbeDone};
use crate::video::media_probe::ProbeResult;
use log::warn;

impl DeliveryWorker {
    pub(crate) async fn finish_chunk(&mut self, done: ChunkDone) {
        self.keeper.note_chunk(&done);
        self.inflight.remove(&done.chunk);
        match done.outcome {
            Ok(result) => self.absorb_chunk(&done.chunk.post, &done.url, result).await,
            Err(error) => {
                warn!("Chunk transfer failed: {error:#}");
                self.start_cooldown(done.chunk.post);
            }
        }
    }

    async fn absorb_chunk(&mut self, post: &PostId, url: &str, result: ChunkResult) {
        self.learn(post, url, result.total_bytes, result.accept_ranges).await;
        if result.bytes_written == 0 && !result.cancelled {
            // The server ignored the range: no progress is possible
            // right now; back off before reclassifying work.
            return self.start_cooldown(post.clone());
        }
        self.try_finalize(post).await;
    }

    pub(crate) async fn finish_probe(&mut self, done: ProbeDone) {
        self.keeper.note_probe(&done);
        self.probes.finished(&done.post);
        match done.outcome {
            Ok(result) => self.absorb_probe(&done.post, &done.url, result).await,
            Err(error) => warn!("Probe failed: {error:#}"),
        }
    }

    async fn absorb_probe(&mut self, post: &PostId, url: &str, result: ProbeResult) {
        self.learn(post, url, result.content_length, result.accept_ranges)
            .await;
    }

    async fn learn(&mut self, post: &PostId, url: &str, total: Option<u64>, ranged: bool) {
        let facts = LearnedFacts {
            content_length: total,
            accept_ranges: Some(ranged),
            host: host_of(url),
        };
        self.state.catalog_mut().learn(post, facts);
        if let Some(total) = total {
            self.set_store_total(post, total).await;
        }
    }

    /// Declares the store total once; a conflicting later fact only
    /// warns — the stored length stays authoritative for serving.
    pub(crate) async fn set_store_total(&self, post: &PostId, total: u64) {
        let key = post.as_str();
        match self.ctx.store.total_len(key).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                if let Err(error) = self.ctx.store.set_total_len(key, total).await {
                    warn!("Total length rejected for {key}: {error:#}");
                }
            }
            Err(error) => warn!("Store lookup failed for {key}: {error:#}"),
        }
    }

    async fn try_finalize(&mut self, post: &PostId) {
        let advertised = self
            .state
            .catalog()
            .lookup(post)
            .and_then(|entry| entry.meta.sha256.clone());
        let Some(digest) = advertised else { return };
        if !self.ctx.store.is_complete(post.as_str()).await.unwrap_or(false) {
            return;
        }
        if let Err(error) = self.ctx.store.finalize(post.as_str(), &digest).await {
            warn!("Finalize failed for {}: {error:#}", post.as_str());
            self.start_cooldown(post.clone());
        }
    }

    pub(crate) fn start_cooldown(&mut self, post: PostId) {
        if !self.cooling.insert(post.clone()) {
            return;
        }
        let events = self.ctx.events.clone();
        let wait = self.tuning.failure_cooldown;
        tokio::spawn(async move {
            tokio::time::sleep(wait).await;
            let _ = events.send(InternalEvent::CooldownOver(post));
        });
    }
}
