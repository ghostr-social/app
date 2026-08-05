//! The pending-retrieval queue and its takeout order: the focused
//! context leaves first, then the more urgent priority class, then
//! submission order.

use std::cmp::Ordering;

use crate::discovery::session_generation::SessionGeneration;

/// Screen-level scope a retrieval serves, e.g. `feed`, `search:ghost`,
/// `tag:dance`, or `discover`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FeedContext {
    value: String,
    session: SessionGeneration,
}

impl FeedContext {
    pub fn new(value: impl Into<String>) -> Self {
        Self::for_session(value, SessionGeneration::initial())
    }

    pub(crate) fn for_session(value: impl Into<String>, session: SessionGeneration) -> Self {
        Self {
            value: value.into(),
            session,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub(crate) fn session(&self) -> SessionGeneration {
        self.session
    }
}

/// Priority classes for content retrieval, most urgent first.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RetrievalPriority {
    Interactive,
    Enrichment,
    Background,
}

/// Describes one unit of network retrieval for scheduling decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalRequest {
    pub context: FeedContext,
    pub priority: RetrievalPriority,
}

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
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            sequence: 0,
            focused: None,
        }
    }

    /// Marks `context` as what the viewer is looking at right now.
    pub fn focus(&mut self, context: FeedContext) {
        self.focused = Some(context);
    }

    pub fn push(&mut self, request: RetrievalRequest, payload: T) {
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

    pub fn has_pending(&self, context: &FeedContext) -> bool {
        self.pending
            .iter()
            .any(|entry| &entry.request.context == context)
    }

    /// Removes and returns the most urgent pending retrieval.
    pub fn take_next(&mut self) -> Option<(RetrievalRequest, T)> {
        let best = self.best_index()?;
        let entry = self.pending.remove(best);
        Some((entry.request, entry.payload))
    }

    fn best_index(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (index, entry) in self.pending.iter().enumerate() {
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
