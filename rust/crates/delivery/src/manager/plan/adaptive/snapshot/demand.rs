use ghostr_engine::adaptive::{CandidateSnapshot, PlayableRange};
use ghostr_engine::ByteRange;

pub(super) fn prioritize(candidate: &mut CandidateSnapshot, wanted: ByteRange) {
    let mut surrounding = Vec::new();
    let mut overlap_gain = 0_u64;
    for playable in std::mem::take(&mut candidate.playable_ranges) {
        let (pieces, gain) = split_around(playable, wanted);
        surrounding.extend(pieces);
        overlap_gain = overlap_gain.saturating_add(gain);
    }
    candidate.playable_ranges.push(PlayableRange {
        bytes: wanted,
        playable_ms: demanded_gain(wanted, candidate.bitrate_bps, overlap_gain),
    });
    candidate.playable_ranges.extend(surrounding);
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}

fn split_around(playable: PlayableRange, wanted: ByteRange) -> (Vec<PlayableRange>, u64) {
    if !overlaps(playable.bytes, wanted) {
        return (vec![playable], 0);
    }
    let overlap = ByteRange::new(
        playable.bytes.start.max(wanted.start),
        playable.bytes.end.min(wanted.end),
    );
    let pieces = [
        piece(playable, playable.bytes.start, overlap.start),
        piece(playable, overlap.end, playable.bytes.end),
    ]
    .into_iter()
    .flatten()
    .collect();
    (pieces, proportional_gain(playable, overlap.len()))
}

fn piece(playable: PlayableRange, start: u64, end: u64) -> Option<PlayableRange> {
    (start < end).then(|| PlayableRange {
        bytes: ByteRange::new(start, end),
        playable_ms: proportional_gain(playable, end - start),
    })
}

fn proportional_gain(playable: PlayableRange, bytes: u64) -> u64 {
    let gain = u128::from(playable.playable_ms).saturating_mul(u128::from(bytes));
    (gain / u128::from(playable.bytes.len().max(1)))
        .max(1)
        .min(u128::from(u64::MAX)) as u64
}

fn demanded_gain(wanted: ByteRange, bitrate_bps: u64, overlap_gain: u64) -> u64 {
    if overlap_gain > 0 {
        return overlap_gain;
    }
    wanted
        .len()
        .saturating_mul(8_000)
        .div_ceil(bitrate_bps.max(1))
        .max(1)
}
