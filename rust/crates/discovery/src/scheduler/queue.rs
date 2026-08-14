//! The pending-retrieval queue and its takeout order: the focused
//! context leaves first, then the more urgent priority class, then
//! submission order.

use std::cmp::Ordering;
use std::collections::HashSet;

use crate::retrieval_types::{FeedContext, RetrievalRequest};

/// Pending retrievals awaiting a worker slot, in submission order;
/// urgency is decided at takeout so a late `focus` still reorders.
#[derive(Debug, Default)]
pub struct RetrievalQueue<T> {
    pending: Vec<Entry<T>>,
    sequence: u64,
    focused: Option<FeedContext>,
}

#[derive(Debug)]
struct Entry<T> {
    request: RetrievalRequest,
    payload: T,
    sequence: u64,
}

impl<T> RetrievalQueue<T> {
    pub(crate) fn new() -> Self {
        Self {
            pending: Vec::new(),
            sequence: 0,
            focused: None,
        }
    }

    /// Marks `context` as what the viewer is looking at right now.
    pub(crate) fn focus(&mut self, context: FeedContext) {
        self.focused = Some(context);
    }

    pub(crate) fn push(&mut self, request: RetrievalRequest, payload: T) {
        self.pending.push(Entry {
            request,
            payload,
            sequence: self.sequence,
        });
        self.sequence += 1;
    }

    pub(crate) fn reset_session(&mut self) {
        self.pending.clear();
        self.focused = None;
    }

    pub(crate) fn remove(&mut self, context: &FeedContext) {
        self.pending
            .retain(|entry| &entry.request.context != context);
        if self.focused.as_ref() == Some(context) {
            self.focused = None;
        }
    }

    pub(crate) fn has_pending(&self, context: &FeedContext) -> bool {
        self.pending
            .iter()
            .any(|entry| &entry.request.context == context)
    }

    /// Removes and returns the most urgent pending retrieval.
    #[cfg(test)]
    pub(crate) fn take_next(&mut self) -> Option<(RetrievalRequest, T)> {
        self.take_next_excluding(std::iter::empty())
    }

    pub(crate) fn take_next_excluding<'a>(
        &mut self,
        blocked: impl IntoIterator<Item = &'a FeedContext>,
    ) -> Option<(RetrievalRequest, T)> {
        let blocked: HashSet<_> = blocked.into_iter().cloned().collect();
        let best = self.best_index(&blocked)?;
        let entry = self.pending.remove(best);
        Some((entry.request, entry.payload))
    }

    fn best_index(&self, blocked: &HashSet<FeedContext>) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (index, entry) in self.pending.iter().enumerate() {
            if blocked.contains(&entry.request.context) {
                continue;
            }
            let beats = match best {
                None => true,
                Some(current) => self.orders_before(entry, &self.pending[current]),
            };
            if beats {
                best = Some(index);
            }
        }
        best
    }

    /// Dart `_ordersBefore`: focused first, then priority, then FIFO.
    fn orders_before(&self, left: &Entry<T>, right: &Entry<T>) -> bool {
        let left_focused = self.is_focused(left);
        if left_focused != self.is_focused(right) {
            return left_focused;
        }
        match left.request.priority.cmp(&right.request.priority) {
            Ordering::Equal => left.sequence < right.sequence,
            order => order == Ordering::Less,
        }
    }

    fn is_focused(&self, entry: &Entry<T>) -> bool {
        self.focused.as_ref() == Some(&entry.request.context)
    }
}
