use super::{DeliveryState, PlanInputs};
use ghostr_engine::media_timeline::normalize;
use ghostr_engine::playback::{AdaptiveBufferPolicy, ContinuationConditions, PlaybackPhase};
use ghostr_engine::{ByteRange, PostId};

pub(super) fn demand(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    post: &PostId,
) -> Option<ByteRange> {
    if state.playback().session()?.post() != post {
        return None;
    }
    let observation = state.playback().observation()?;
    if !matches!(
        observation.phase(),
        PlaybackPhase::Starting | PlaybackPhase::Playing | PlaybackPhase::NetworkStalled
    ) {
        return None;
    }
    let timeline = state.catalog().lookup(post)?.timeline()?;
    let network =
        crate::manager::quality::network_for(state, inputs.stats, post, inputs.observed_at_ms)?;
    let present = normalize(inputs.present.get(post).cloned().unwrap_or_default());
    let target = AdaptiveBufferPolicy::default().target_for_timeline(
        timeline,
        ContinuationConditions {
            observation,
            network,
        },
        &present,
    )?;
    if observation.buffer_ahead() >= target.required() {
        return None;
    }
    let start = observation.position().as_millis().min(u128::from(u64::MAX)) as u64;
    let end = start
        .saturating_add(target.required().as_millis().min(u128::from(u64::MAX)) as u64)
        .min(timeline.selected_end_ms()?);
    let dependencies = timeline.continuation_dependencies(start, end)?;
    dependencies
        .into_iter()
        .find_map(|span| first_missing(span, &present))
}

fn first_missing(span: ByteRange, present: &[ByteRange]) -> Option<ByteRange> {
    let mut cursor = span.start;
    for known in present
        .iter()
        .filter(|known| known.start < span.end && known.end > span.start)
    {
        if known.start > cursor {
            return Some(slice(cursor, known.start.min(span.end)));
        }
        cursor = cursor.max(known.end);
    }
    (cursor < span.end).then(|| slice(cursor, span.end))
}

fn slice(start: u64, end: u64) -> ByteRange {
    ByteRange::new(
        start,
        end.min(start.saturating_add(ghostr_engine::playback::PLAYBACK_SLICE_BYTES)),
    )
}
