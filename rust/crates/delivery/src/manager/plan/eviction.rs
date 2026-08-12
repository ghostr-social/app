//! Decides whether urgent ordering also authorizes destructive seed eviction.

use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::ByteRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtectedSeedEviction {
    Defer,
    Allow,
}

#[derive(Clone, Copy)]
pub(crate) struct EvictionInputs<'a> {
    pub(crate) gateway_demand: bool,
    pub(crate) current_startable: bool,
    pub(crate) demanded: Option<ByteRange>,
    pub(crate) present: &'a [ByteRange],
    pub(crate) phase: Option<PlaybackPhase>,
    pub(crate) playback_emergency: bool,
    pub(crate) buffer_below_emergency: bool,
}

pub(crate) fn protected_seed_eviction(inputs: EvictionInputs<'_>) -> ProtectedSeedEviction {
    let critical = inputs.buffer_below_emergency || playback_critical(inputs);
    let safe = inputs.gateway_demand
        && inputs.current_startable
        && contiguous_read_ahead(inputs.demanded, inputs.present)
        && !critical;
    match safe {
        true => ProtectedSeedEviction::Defer,
        false => ProtectedSeedEviction::Allow,
    }
}

fn playback_critical(inputs: EvictionInputs<'_>) -> bool {
    matches!(inputs.phase, Some(PlaybackPhase::NetworkStalled))
        || (matches!(inputs.phase, Some(PlaybackPhase::Playing)) && inputs.playback_emergency)
}

fn contiguous_read_ahead(demanded: Option<ByteRange>, present: &[ByteRange]) -> bool {
    demanded.is_some_and(|range| range.start == contiguous_prefix_end(present))
}

fn contiguous_prefix_end(present: &[ByteRange]) -> u64 {
    let mut ranges = present.to_vec();
    ranges.sort_by_key(|range| (range.start, range.end));
    let mut end = 0;
    for range in ranges {
        if range.start > end {
            break;
        }
        end = end.max(range.end);
    }
    end
}
