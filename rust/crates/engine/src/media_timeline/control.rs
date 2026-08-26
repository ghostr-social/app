use core::sync::atomic::{AtomicBool, Ordering};

/// Pure cancellation boundary for bounded container parsing.
pub trait TimelineParseControl: Sync {
    fn is_cancelled(&self) -> bool;
}

impl TimelineParseControl for AtomicBool {
    fn is_cancelled(&self) -> bool {
        self.load(Ordering::Acquire)
    }
}
