use core::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

const ACTIVE: u8 = 0;
const REVOKED: u8 = 1;
const PUBLISHED: u8 = 2;

#[derive(Clone, Debug)]
pub(super) struct ActionState(Arc<AtomicU8>);

impl ActionState {
    pub(super) fn new() -> Self {
        Self(Arc::new(AtomicU8::new(ACTIVE)))
    }

    pub(super) fn is_active(&self) -> bool {
        self.0.load(Ordering::Acquire) == ACTIVE
    }

    pub(super) fn revoke(&self) -> bool {
        self.transition(REVOKED)
    }

    pub(super) fn claim_publication(&self) -> bool {
        self.transition(PUBLISHED)
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    fn transition(&self, target: u8) -> bool {
        self.0
            .compare_exchange(ACTIVE, target, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}
