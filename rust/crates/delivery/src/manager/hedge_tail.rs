use crate::manager::transfers::InternalEvent;
use core::time::Duration;
use ghostr_engine::ActionId;
use std::collections::BTreeMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HedgeTailWake {
    action: ActionId,
    deadline_ms: u64,
}

impl HedgeTailWake {
    pub(crate) const fn new(action: ActionId, deadline_ms: u64) -> Self {
        Self {
            action,
            deadline_ms,
        }
    }
}

struct ActiveTimer {
    wake: HedgeTailWake,
    handle: JoinHandle<()>,
}

#[derive(Default)]
pub(crate) struct HedgeTailTimers {
    active: BTreeMap<ActionId, ActiveTimer>,
}

impl HedgeTailTimers {
    pub(crate) fn reconcile(
        &mut self,
        deadlines: &[HedgeTailWake],
        observed_at_ms: u64,
        events: &UnboundedSender<InternalEvent>,
    ) {
        self.retain(deadlines);
        for wake in deadlines {
            if !self.active.contains_key(&wake.action) {
                let wait = Duration::from_millis(wake.deadline_ms.saturating_sub(observed_at_ms));
                self.start(*wake, wait, events.clone());
            }
        }
    }

    pub(crate) fn finish(&mut self, wake: HedgeTailWake) -> bool {
        if self.active.get(&wake.action).map(|timer| timer.wake) != Some(wake) {
            return false;
        }
        self.active.remove(&wake.action);
        true
    }

    pub(super) fn clear(&mut self) {
        for (_, timer) in core::mem::take(&mut self.active) {
            timer.handle.abort();
        }
    }

    fn retain(&mut self, deadlines: &[HedgeTailWake]) {
        self.active.retain(|action, timer| {
            let keep = deadlines
                .iter()
                .any(|wake| wake.action == *action && *wake == timer.wake);
            if !keep {
                timer.handle.abort();
            }
            keep
        });
    }

    fn start(
        &mut self,
        wake: HedgeTailWake,
        wait: Duration,
        events: UnboundedSender<InternalEvent>,
    ) {
        let handle = tokio::spawn(async move {
            tokio::time::sleep(wait).await;
            let _ = events.send(InternalEvent::HedgeTail(wake));
        });
        self.active
            .insert(wake.action, ActiveTimer { wake, handle });
    }
}

impl Drop for HedgeTailTimers {
    fn drop(&mut self) {
        self.clear();
    }
}

impl crate::manager::DeliveryWorker {
    pub(super) fn schedule_hedge_tail_wakes(
        &mut self,
        deadlines: &[HedgeTailWake],
        observed_at_ms: u64,
    ) {
        self.hedge_tail_timers
            .reconcile(deadlines, observed_at_ms, &self.ctx.events);
    }

    pub(super) fn consume_hedge_tail_wake(&mut self, wake: HedgeTailWake) {
        self.hedge_tail_timers.finish(wake);
    }
}
