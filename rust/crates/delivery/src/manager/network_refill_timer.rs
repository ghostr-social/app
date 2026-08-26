use crate::manager::transfers::InternalEvent;
use core::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NetworkRefillWake {
    generation: NetworkRefillGeneration,
    deadline_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NetworkRefillGeneration(u128);

impl NetworkRefillWake {
    const fn new(generation: u128, deadline_ms: u64) -> Self {
        Self {
            generation: NetworkRefillGeneration(generation),
            deadline_ms,
        }
    }
}

struct ActiveTimer {
    wake: NetworkRefillWake,
    handle: JoinHandle<()>,
}

#[derive(Default)]
pub(crate) struct NetworkRefillTimer {
    active: Option<ActiveTimer>,
    next_generation: u128,
}

impl NetworkRefillTimer {
    pub(crate) fn reconcile(
        &mut self,
        deadline_ms: Option<u64>,
        observed_at_ms: u64,
        events: &UnboundedSender<InternalEvent>,
    ) -> Option<NetworkRefillWake> {
        if self.active.as_ref().map(|timer| timer.wake.deadline_ms) == deadline_ms {
            return self.active.as_ref().map(|timer| timer.wake);
        }
        self.clear();
        let deadline_ms = deadline_ms?;
        let wake = NetworkRefillWake::new(self.next_generation, deadline_ms);
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .expect("network-refill wake generation exhausted");
        let wait = Duration::from_millis(deadline_ms.saturating_sub(observed_at_ms));
        let events = events.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(wait).await;
            let _ = events.send(InternalEvent::NetworkRefill(wake));
        });
        self.active = Some(ActiveTimer { wake, handle });
        Some(wake)
    }

    pub(crate) fn reconcile_now(
        &mut self,
        deadline_ms: Option<u64>,
        events: &UnboundedSender<InternalEvent>,
    ) -> Option<NetworkRefillWake> {
        self.reconcile(deadline_ms, crate::manager::time::unix_time_ms(), events)
    }

    pub(crate) fn finish(&mut self, wake: NetworkRefillWake) -> bool {
        if self.active.as_ref().map(|timer| timer.wake) != Some(wake) {
            return false;
        }
        self.active.take();
        true
    }

    pub(super) fn clear(&mut self) {
        if let Some(timer) = self.active.take() {
            timer.handle.abort();
        }
    }
}

impl Drop for NetworkRefillTimer {
    fn drop(&mut self) {
        self.clear();
    }
}

impl crate::manager::DeliveryWorker {
    pub(super) fn schedule_network_refill_wake(&mut self, deadline_ms: Option<u64>) {
        self.network_refill_timer
            .reconcile_now(deadline_ms, &self.ctx.events);
    }
}
