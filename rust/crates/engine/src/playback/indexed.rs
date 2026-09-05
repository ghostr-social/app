//! Serial, dependency-complete continuation on a conservative share of the path.
//! These conditional scenarios never turn byte counts into decoder readiness.
use super::{
    AdaptiveBufferPolicy, BufferScenario, BufferTarget, NetworkConditions, PlaybackObservation,
    UsableArrival,
};
use crate::media_timeline::{normalize, MediaTimeline};
use crate::ByteRange;
use core::time::Duration;

#[derive(Clone, Copy)]
pub struct ContinuationConditions {
    pub observation: PlaybackObservation,
    pub network: NetworkConditions,
}

struct IndexedScenario<'a> {
    timeline: &'a MediaTimeline,
    conditions: ContinuationConditions,
    present: Vec<ByteRange>,
    beginning: u64,
    horizon: u64,
}

impl AdaptiveBufferPolicy {
    pub fn target_for_timeline(
        self,
        timeline: &MediaTimeline,
        conditions: ContinuationConditions,
        present: &[ByteRange],
    ) -> Option<BufferTarget> {
        let position = millis(conditions.observation.position());
        let remaining = timeline.selected_end_ms()?.saturating_sub(position);
        let horizon = remaining
            .saturating_mul(1_000)
            .div_ceil(u64::from(conditions.observation.playback_rate_milli()))
            .min(20_000);
        let input = IndexedScenario {
            timeline,
            conditions,
            present: normalize(present.to_vec()),
            beginning: position.saturating_add(millis(conditions.observation.buffer_ahead())),
            horizon,
        };
        let arrivals = input.arrivals()?;
        let scenario = BufferScenario::new(
            horizon,
            conditions.observation.playback_rate_milli(),
            conditions.observation.phase(),
        );
        let required = Duration::from_millis(scenario.required_ms(&arrivals).ok()?)
            .max(self.minimum.min(Duration::from_millis(remaining)));
        Some(self.with_requirement(required, conditions.network))
    }
}

impl IndexedScenario<'_> {
    fn arrivals(&self) -> Option<Vec<UsableArrival>> {
        let end = self.timeline.selected_end_ms()?;
        let mut arrivals = Vec::with_capacity(20);
        let rate = u64::from(self.conditions.observation.playback_rate_milli());
        let media_horizon = self.horizon.saturating_mul(rate).div_ceil(1_000);
        for step in 1..=media_horizon.div_ceil(1_000).min(120) {
            let frontier = self.beginning.saturating_add(step * 1_000).min(end);
            if frontier <= self.beginning {
                break;
            }
            let dependencies = self
                .timeline
                .continuation_dependencies(self.beginning, frontier)?;
            let bytes = dependencies.iter().map(|span| self.missing(*span)).sum();
            arrivals.push(UsableArrival::new(
                completion_ms(self.conditions.network, bytes),
                frontier - self.beginning,
            ));
            if frontier == end {
                break;
            }
        }
        Some(arrivals)
    }

    fn missing(&self, span: ByteRange) -> u64 {
        let covered: u64 = self
            .present
            .iter()
            .map(|known| {
                span.end
                    .min(known.end)
                    .saturating_sub(span.start.max(known.start))
            })
            .sum();
        span.len().saturating_sub(covered)
    }
}

fn completion_ms(network: NetworkConditions, bytes: u64) -> u64 {
    if bytes == 0 {
        return 0;
    }
    // Two protected items share the path; neither is assigned all unused service.
    let rate = network.sustainable_bits_per_second() / 2;
    if rate == 0 {
        return u64::MAX;
    }
    let transfer = u128::from(bytes)
        .saturating_mul(8_000)
        .div_ceil(u128::from(rate));
    let requests = bytes.div_ceil(super::PLAYBACK_SLICE_BYTES);
    let delay = millis(network.ttfb).saturating_mul(requests);
    (transfer.min(u128::from(u64::MAX)) as u64)
        .saturating_add(delay)
        .saturating_add(super::continuation::processing_margin_ms(
            network.confidence,
        ))
}

fn millis(value: Duration) -> u64 {
    value.as_millis().min(u128::from(u64::MAX)) as u64
}
