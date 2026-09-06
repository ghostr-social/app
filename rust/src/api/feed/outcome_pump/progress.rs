//! Coalesce only already-received, consecutive progress for one feed/session.
//! A context change or lifecycle outcome is a barrier, never reordered.

use super::{lock, Event, FeedContext, OutcomeSinks, ReadyOutcomes, RetrievalOutcome};

pub(super) fn apply(
    sinks: &OutcomeSinks,
    context: &FeedContext,
    first: Event,
    ready: &mut ReadyOutcomes,
) {
    let events = collect(context, first, ready);
    let candidates = lock(&sinks.state).apply_progress(context, &events);
    for candidate in candidates {
        crate::api::delivery::candidates::admit(sinks.candidates.as_ref(), Some(candidate));
    }
}

fn collect(context: &FeedContext, first: Event, ready: &mut ReadyOutcomes) -> Vec<Event> {
    let mut events = vec![first];
    while matches!(ready.peek(), Some(RetrievalOutcome::Progress { context: next, .. })
        if next == context)
    {
        if let Some(RetrievalOutcome::Progress { event, .. }) = ready.next() {
            events.push(*event);
        }
    }
    events
}
