//! Completion policy for media-size probes.

use crate::engine::PostId;
use crate::video::delivery_failure::{classify, FailureClass};
use crate::video::delivery_manager::DeliveryWorker;
use crate::video::delivery_retry::Source;
use crate::video::delivery_transfers::ProbeDone;
use crate::video::media_probe::ProbeResult;
use log::warn;

impl DeliveryWorker {
    pub(crate) async fn finish_probe(&mut self, done: ProbeDone) {
        if self.state.catalog().lookup(&done.post).is_none() {
            self.probes.release(&done.post);
            return;
        }
        self.keeper.note_probe(&done);
        match done.outcome {
            Ok(result) => {
                self.finish_probe_result(&done.post, &done.url, result)
                    .await
            }
            Err(error) => self.finish_probe_error(&done.post, &done.url, error),
        }
    }

    async fn finish_probe_result(&mut self, post: &PostId, url: &str, result: ProbeResult) {
        if result.content_length.is_some_and(|length| length > 0) {
            self.probes.learned(post);
            self.absorb_probe(post, url, result).await;
            return;
        }
        self.probes.release(post);
        warn!("Probe did not reveal a usable content length for {url}");
        self.note_failed_attempt(post, url, FailureClass::Transient);
    }

    fn finish_probe_error(&mut self, post: &PostId, url: &str, error: anyhow::Error) {
        self.probes.release(post);
        warn!("Probe failed: {error:#}");
        self.note_failed_attempt(post, url, classify(&error));
    }

    async fn absorb_probe(&mut self, post: &PostId, url: &str, result: ProbeResult) {
        self.retry
            .note_success(&Source::new(post.clone(), url.to_owned()));
        self.learn(post, url, result.content_length, result.accept_ranges)
            .await;
    }
}
