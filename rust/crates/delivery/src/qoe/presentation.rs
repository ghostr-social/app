use super::{ActivePlayback, QoeStats, QoeTracker};
use ghostr_engine::PostId;

pub(super) struct FinishedPlayback {
    post: PostId,
    focused_at_ms: u64,
    first_frame: bool,
}

impl FinishedPlayback {
    pub(super) fn from_active(active: &ActivePlayback) -> Self {
        Self {
            post: active.post.clone(),
            focused_at_ms: active.focused_at_ms,
            first_frame: active.first_frame,
        }
    }
}

impl QoeTracker {
    pub fn present(&mut self, post: &PostId, now_ms: u64) {
        if let Some(active) = self.active.as_mut().filter(|active| &active.post == post) {
            record_first_frame(
                &mut self.stats,
                active.focused_at_ms,
                &mut active.first_frame,
                now_ms,
            );
            return;
        }
        let Some(recent) = self.recent.as_mut().filter(|recent| &recent.post == post) else {
            return;
        };
        if recent.first_frame {
            return;
        }
        self.stats.startup_failures = self.stats.startup_failures.saturating_sub(1);
        record_first_frame(
            &mut self.stats,
            recent.focused_at_ms,
            &mut recent.first_frame,
            now_ms,
        );
    }
}

fn record_first_frame(
    stats: &mut QoeStats,
    focused_at_ms: u64,
    first_frame: &mut bool,
    now_ms: u64,
) {
    if *first_frame {
        return;
    }
    *first_frame = true;
    let startup = now_ms.saturating_sub(focused_at_ms);
    stats.first_frames += 1;
    stats.startup_total_ms = stats.startup_total_ms.saturating_add(startup);
    stats.startup_max_ms = stats.startup_max_ms.max(startup);
}
