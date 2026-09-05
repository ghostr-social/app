use super::attempts::TimelineAttempt;
use super::outcome::{TimelineJobOutcome, TimelineResult, TimelineRetry};
use super::parser::{TimelineInput, TimelineParse, TimelineParser};
use super::TimelineEvidence;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, RepresentationRead};
use std::sync::Arc;

mod indexed;

pub(super) async fn run(
    attempt: TimelineAttempt,
    store: Arc<PartialRangeStore>,
    parser: Arc<dyn TimelineParser>,
) -> TimelineResult {
    let outcome = indexed::run(&attempt, &store, parser).await;
    TimelineResult::new(attempt, outcome)
}

async fn compile(
    attempt: &TimelineAttempt,
    store: &PartialRangeStore,
    parser: Arc<dyn TimelineParser>,
) -> TimelineJobOutcome {
    match load_input(attempt, store).await {
        Ok(input) if !attempt.is_cancelled() => {
            let parsed = parse(input, parser, attempt.control()).await;
            if attempt.is_cancelled() {
                TimelineJobOutcome::Superseded
            } else {
                parsed
            }
        }
        Ok(_) => TimelineJobOutcome::Superseded,
        Err(outcome) => outcome,
    }
}

async fn load_input(
    attempt: &TimelineAttempt,
    store: &PartialRangeStore,
) -> Result<TimelineInput, TimelineJobOutcome> {
    let evidence = attempt.evidence();
    let mut segments = Vec::with_capacity(evidence.spans().len());
    for span in evidence.spans() {
        if attempt.is_cancelled() {
            return Err(TimelineJobOutcome::Superseded);
        }
        let read = read_span(store, evidence, span.start..span.end).await?;
        segments.push((span.start, read));
    }
    ensure_evidence(store, evidence).await?;
    Ok(TimelineInput::new(evidence.total(), segments))
}

async fn read_span(
    store: &PartialRangeStore,
    evidence: &TimelineEvidence,
    span: core::ops::Range<u64>,
) -> Result<Vec<u8>, TimelineJobOutcome> {
    let read = store
        .read_for_stream(
            evidence.binding().post().as_str(),
            Some(evidence.binding()),
            evidence.revision(),
            span,
        )
        .await
        .map_err(|error| TimelineJobOutcome::Retryable(TimelineRetry::Read(error.to_string())))?;
    match read {
        RepresentationRead::Present(bytes) => Ok(bytes),
        RepresentationRead::Missing => Err(TimelineJobOutcome::Retryable(TimelineRetry::Missing)),
        RepresentationRead::Superseded => Err(TimelineJobOutcome::Superseded),
    }
}

async fn ensure_evidence(
    store: &PartialRangeStore,
    expected: &TimelineEvidence,
) -> Result<(), TimelineJobOutcome> {
    let snapshot = store
        .media_snapshot(expected.binding().post().as_str())
        .await
        .map_err(|error| TimelineJobOutcome::Retryable(TimelineRetry::Read(error.to_string())))?;
    expected
        .still_valid_in(&snapshot)
        .then_some(())
        .ok_or(TimelineJobOutcome::Superseded)
}

async fn parse(
    input: TimelineInput,
    parser: Arc<dyn TimelineParser>,
    control: Arc<core::sync::atomic::AtomicBool>,
) -> TimelineJobOutcome {
    match tokio::task::spawn_blocking(move || parser.parse(input, control.as_ref())).await {
        Ok(TimelineParse::Cancelled) => TimelineJobOutcome::Superseded,
        Ok(TimelineParse::Completed(terminal)) => TimelineJobOutcome::Terminal(terminal),
        Err(error) => TimelineJobOutcome::Retryable(TimelineRetry::Worker(error.to_string())),
    }
}
