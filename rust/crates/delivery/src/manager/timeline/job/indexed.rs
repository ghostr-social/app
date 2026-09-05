use super::{TimelineAttempt, TimelineJobOutcome, TimelineParser};
use crate::manager::timeline::TimelineTerminal;
use ghostr_partial_store::partial_range_store::{CompiledIndexKey, PartialRangeStore};
use std::sync::Arc;

pub(super) async fn run(
    attempt: &TimelineAttempt,
    store: &PartialRangeStore,
    parser: Arc<dyn TimelineParser>,
) -> TimelineJobOutcome {
    if let Some(outcome) = cached(attempt, store).await {
        return outcome;
    }
    let outcome = super::compile(attempt, store, parser).await;
    if let TimelineJobOutcome::Terminal(TimelineTerminal::Ready(timeline)) = &outcome {
        if let Some(key) = current_key(attempt, store).await {
            // Retention is best effort; storage pressure cannot fail an otherwise
            // valid structural parse or evict protected payload to save an index.
            let _ = store.retain_compiled_index(&key, timeline).await;
        }
    }
    outcome
}

async fn cached(
    attempt: &TimelineAttempt,
    store: &PartialRangeStore,
) -> Option<TimelineJobOutcome> {
    let key = current_key(attempt, store).await?;
    let timeline = store.compiled_index(&key).await.ok()??;
    current_key(attempt, store).await?;
    Some(TimelineJobOutcome::Terminal(TimelineTerminal::Ready(
        Box::new(timeline),
    )))
}

async fn current_key(
    attempt: &TimelineAttempt,
    store: &PartialRangeStore,
) -> Option<CompiledIndexKey> {
    let evidence = attempt.evidence();
    let (identity, source) = evidence.source.as_ref()?;
    if attempt.is_cancelled() || !store.http_generation_matches_source(identity, source).await {
        return None;
    }
    super::ensure_evidence(store, evidence).await.ok()?;
    Some(CompiledIndexKey::native_mp4(
        evidence.binding().representation(),
        source,
    ))
}
