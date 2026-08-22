use std::sync::atomic::{AtomicBool, Ordering};

/// Pure cancellation boundary for bounded container parsing.
pub trait TimelineParseControl: Sync {
    fn is_cancelled(&self) -> bool;
}

pub(super) struct NeverCancelled;

impl TimelineParseControl for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

impl TimelineParseControl for AtomicBool {
    fn is_cancelled(&self) -> bool {
        self.load(Ordering::Acquire)
    }
}
