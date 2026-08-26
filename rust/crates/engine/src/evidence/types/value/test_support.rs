use super::*;

impl<T> Evidence<T> {
    pub(crate) fn invalidated_at_ms(&self) -> Option<u64> {
        self.invalidated_at_ms
    }
}
