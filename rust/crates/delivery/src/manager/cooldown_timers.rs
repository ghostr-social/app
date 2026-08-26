use crate::manager::retry::CooldownId;
use crate::manager::transfers::{InternalEvent, MaintenanceEvent};
use core::time::Duration;
use ghostr_engine::PostId;
use std::collections::{BTreeMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

struct ActiveTimer {
    cooldown: CooldownId,
    handle: JoinHandle<()>,
}

#[derive(Default)]
pub(crate) struct CooldownTimers {
    active: BTreeMap<PostId, ActiveTimer>,
}

impl CooldownTimers {
    pub(crate) fn start(
        &mut self,
        post: PostId,
        cooldown: CooldownId,
        wait: Duration,
        events: UnboundedSender<InternalEvent>,
    ) {
        let event_post = post.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(wait).await;
            let event = MaintenanceEvent::CooldownOver(event_post, cooldown);
            let _ = events.send(InternalEvent::Maintenance(event));
        });
        let timer = ActiveTimer { cooldown, handle };
        if let Some(replaced) = self.active.insert(post, timer) {
            replaced.handle.abort();
        }
    }

    pub(crate) fn cancel(&mut self, post: &PostId) {
        if let Some(timer) = self.active.remove(post) {
            timer.handle.abort();
        }
    }

    pub(crate) fn finish(&mut self, post: &PostId, cooldown: CooldownId) -> bool {
        if self.active.get(post).map(|timer| timer.cooldown) != Some(cooldown) {
            return false;
        }
        self.active.remove(post);
        true
    }

    pub(crate) fn retain(&mut self, retained: &HashSet<PostId>) {
        self.active.retain(|post, timer| {
            let keep = retained.contains(post);
            if !keep {
                timer.handle.abort();
            }
            keep
        });
    }

    pub(crate) fn clear(&mut self) {
        for (_, timer) in core::mem::take(&mut self.active) {
            timer.handle.abort();
        }
    }
}

impl Drop for CooldownTimers {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
#[path = "cooldown_timers_axiom_test.rs"]
pub(crate) mod axiom_test_support;
