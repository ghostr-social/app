use super::super::EvaluationTracker;
use crate::evaluation::events::TransferMetricEvent;
use ghostr_engine::PostId;

impl EvaluationTracker {
    pub(crate) fn transfer(&mut self, event: TransferMetricEvent) {
        if event.full_download_started {
            self.note_unused_full_download(event.post);
        }
        let metrics = &mut self.metrics.efficiency;
        metrics.total_bytes = metrics.total_bytes.saturating_add(event.total_bytes);
        metrics.aborted_bytes = metrics.aborted_bytes.saturating_add(event.aborted_bytes);
        metrics.duplicate_hedge_bytes = metrics
            .duplicate_hedge_bytes
            .saturating_add(event.duplicate_hedge_bytes);
        metrics.completable_probe_bytes = metrics
            .completable_probe_bytes
            .saturating_add(event.completable_probe_bytes);
        metrics.request_count += u64::from(event.request_started);
        metrics.connection_restarts_avoided_by_promotion +=
            u64::from(event.promotion_avoided_restart);
        metrics.cpu_micros = metrics.cpu_micros.saturating_add(event.cpu_micros);
        metrics.storage_byte_ms = metrics
            .storage_byte_ms
            .saturating_add(event.storage_byte_ms);
    }

    fn note_unused_full_download(&mut self, post: Option<PostId>) {
        const CAPACITY: usize = 256;
        let Some(post) = post else {
            self.metrics.efficiency.full_downloads_never_useful += 1;
            return;
        };
        let useful = self
            .active
            .as_ref()
            .is_some_and(|active| active.post == post && active.presented);
        if !useful && self.unused_full_downloads.len() < CAPACITY {
            self.unused_full_downloads.insert(post);
        } else if !useful && !self.unused_full_downloads.contains(&post) {
            self.metrics.efficiency.full_downloads_never_useful += 1;
        }
    }
}
