//! Absorbing finished IO: results teach the catalog and the store,
//! failures are charged to the per-source retry policy so the event
//! loop cannot hammer a broken source, and complete files are
//! finalized.

use crate::engine::catalog::LearnedFacts;
use crate::engine::host_stats::host_of;
use crate::engine::PostId;
use crate::video::chunk_downloader::ChunkResult;
use crate::video::delivery_failure::{classify, FailureClass};
use crate::video::delivery_manager::DeliveryWorker;
use crate::video::delivery_retry::{Retry, Source};
use crate::video::delivery_transfers::{ChunkDone, InternalEvent, ProbeDone};
use crate::video::media_probe::ProbeResult;
use log::warn;
use std::time::Duration;

impl DeliveryWorker {
    pub(crate) async fn finish_chunk(&mut self, done: ChunkDone) {
        self.keeper.note_chunk(&done);
        self.inflight.remove(&done.chunk);
        // Free space moves while a chunk is in flight — other apps write
        // to the same volume — so every finished chunk re-measures it and
        // evicts down to the cap instead of waiting for the next write.
        self.ctx.store.enforce_capacity().await;
        match done.outcome {
            Ok(result) => self.absorb_chunk(&done.chunk.post, &done.url, result).await,
            Err(error) => {
                warn!("Chunk transfer failed: {error:#}");
                self.note_failed_attempt(&done.chunk.post, &done.url, classify(&error));
            }
        }
    }

    async fn absorb_chunk(&mut self, post: &PostId, url: &str, result: ChunkResult) {
        self.learn(post, url, result.total_bytes, result.accept_ranges).await;
        if result.bytes_written == 0 && !result.cancelled {
            // The server ignored the range: no progress is possible
            // right now, so it counts as a failed attempt.
            return self.note_failed_attempt(post, url, FailureClass::Transient);
        }
        self.retry.note_success(&Source::new(post.clone(), url.to_owned()));
        self.try_finalize(post, url).await;
    }

    pub(crate) async fn finish_probe(&mut self, done: ProbeDone) {
        self.keeper.note_probe(&done);
        self.probes.finished(&done.post);
        match done.outcome {
            Ok(result) => self.absorb_probe(&done.post, &done.url, result).await,
            Err(error) => {
                warn!("Probe failed: {error:#}");
                self.note_failed_attempt(&done.post, &done.url, classify(&error));
            }
        }
    }

    async fn absorb_probe(&mut self, post: &PostId, url: &str, result: ProbeResult) {
        self.retry.note_success(&Source::new(post.clone(), url.to_owned()));
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

    /// Byte-complete files leave the partial pool whether or not the
    /// note advertised an `imeta x`; an advertised digest still decides
    /// whether the bytes are kept (see `partial_range_completion`).
    /// Bytes that fail that check came from the source, so a failed
    /// finalize is charged to it like any other failed attempt.
    async fn try_finalize(&mut self, post: &PostId, url: &str) {
        if !self.ctx.store.is_complete(post.as_str()).await.unwrap_or(false) {
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

    /// Charges one failed attempt to the source: either a paced retry
    /// with the policy's backoff, or retirement of the source, which
    /// falls back to the post's other mirrors.
    fn note_failed_attempt(&mut self, post: &PostId, url: &str, class: FailureClass) {
        let source = Source::new(post.clone(), url.to_owned());
        match self.retry.note_failure(source, class) {
            Retry::After(wait) => self.start_cooldown(post.clone(), wait),
            Retry::GiveUp => self.retire_source(post, url),
        }
    }

    fn retire_source(&mut self, post: &PostId, url: &str) {
        let id = post.as_str();
        if self.is_servable(post) {
            warn!("Giving up on {url} for {id}; another source remains");
            return;
        }
        warn!("No working source left for {id}; reporting it unplayable");
        self.refresh_servable();
    }

    /// The gateway may only advertise posts that something can still be
    /// fetched for: an exhausted post answers 404 instead of stalling
    /// the player on bytes that will never arrive.
    pub(crate) fn refresh_servable(&self) {
        let live = self
            .state
            .window_posts()
            .into_iter()
            .filter(|post| self.is_servable(post))
            .map(|post| post.0);
        self.posts.replace_all(live);
    }

    /// Catalogued (therefore progressive) and with a live source left.
    fn is_servable(&self, post: &PostId) -> bool {
        self.state
            .catalog()
            .lookup(post)
            .is_some_and(|entry| !self.retry.all_retired(post, &entry.meta.urls))
    }

    fn start_cooldown(&mut self, post: PostId, wait: Duration) {
        if !self.retry.cool_down(post.clone()) {
            return;
        }
        let events = self.ctx.events.clone();
        tokio::spawn(async move {
            tokio::time::sleep(wait).await;
            let _ = events.send(InternalEvent::CooldownOver(post));
        });
    }
}
