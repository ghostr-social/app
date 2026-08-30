use core::sync::atomic::{AtomicU8, Ordering};

const CANCEL_REQUESTED: u8 = 1;
const IO_FINISHED: u8 = 2;
const HEDGE_AUTHORIZED: u8 = 4;

#[derive(Debug, Default)]
pub(super) struct AttemptLifecycle {
    phase: AtomicU8,
}

impl AttemptLifecycle {
    pub(super) fn mark_io_finished(&self) {
        self.phase.fetch_or(IO_FINISHED, Ordering::AcqRel);
    }

    pub(super) fn io_finished(&self) -> bool {
        self.has(IO_FINISHED)
    }

    pub(super) fn begin_cancel(&self) -> bool {
        self.set_unless(CANCEL_REQUESTED | IO_FINISHED, CANCEL_REQUESTED)
    }

    pub(super) fn authorize_hedge(&self) -> bool {
        self.set_unless(
            CANCEL_REQUESTED | IO_FINISHED | HEDGE_AUTHORIZED,
            HEDGE_AUTHORIZED,
        )
    }

    pub(super) fn hedge_authorized(&self) -> bool {
        self.has(HEDGE_AUTHORIZED)
    }

    pub(super) fn release_hedge(&self) {
        self.phase.fetch_and(!HEDGE_AUTHORIZED, Ordering::AcqRel);
    }

    fn has(&self, flag: u8) -> bool {
        self.phase.load(Ordering::Acquire) & flag != 0
    }

    fn set_unless(&self, blocked: u8, target: u8) -> bool {
        self.phase
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |phase| {
                (phase & blocked == 0).then_some(phase | target)
            })
            .is_ok()
    }
}
