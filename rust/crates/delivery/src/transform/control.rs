use anyhow::{ensure, Result};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct TransformControl {
    phase: Arc<AtomicU8>,
    deadline: Instant,
}

#[repr(u8)]
enum TransformPhase {
    Running,
    Cancelled,
    Committing,
}

impl TransformControl {
    pub fn new(deadline: Instant) -> Self {
        Self {
            phase: Arc::new(AtomicU8::new(TransformPhase::Running as u8)),
            deadline,
        }
    }

    pub fn checkpoint(&self) -> Result<()> {
        ensure!(
            self.phase.load(Ordering::Acquire) == TransformPhase::Running as u8,
            "transform cancelled"
        );
        ensure!(
            Instant::now() <= self.deadline,
            "transform deadline exceeded"
        );
        Ok(())
    }

    pub fn cancel(&self) -> bool {
        self.transition(TransformPhase::Cancelled)
    }

    pub(crate) fn try_begin_commit(&self) -> bool {
        if Instant::now() > self.deadline {
            self.cancel();
            return false;
        }
        self.transition(TransformPhase::Committing)
    }

    fn transition(&self, next: TransformPhase) -> bool {
        self.phase
            .compare_exchange(
                TransformPhase::Running as u8,
                next as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
}
