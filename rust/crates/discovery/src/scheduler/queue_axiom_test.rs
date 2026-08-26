use super::*;

impl<T> RetrievalQueue<T> {
    /// Removes and returns the most urgent pending retrieval.
    pub(crate) fn take_next(&mut self) -> Option<(RetrievalRequest, T)> {
        self.take_next_excluding(core::iter::empty())
    }
}
