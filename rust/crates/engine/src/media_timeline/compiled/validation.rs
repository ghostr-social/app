use super::super::{timing::PresentationTime, MediaTimeline, TimedRange, TimelineError};
use crate::ByteRange;

pub(super) fn validate(timeline: &MediaTimeline, total: u64) -> Result<(), TimelineError> {
    let ranges = timeline
        .inspected
        .iter()
        .chain(&timeline.metadata)
        .chain(&timeline.file_types)
        .chain(timeline.movie.iter());
    let valid = total > 0
        && ranges.copied().all(|range| within(range, total))
        && timeline.media.len() <= 200_000
        && timeline.media_data.len() <= 4_096;
    if !valid || !valid_media_data(timeline, total) {
        return Err(TimelineError::Malformed);
    }
    validate_tracks(&timeline.media, total)
}

fn valid_media_data(timeline: &MediaTimeline, total: u64) -> bool {
    timeline.media_data.iter().all(|data| {
        within(data.header, total)
            && within(data.payload, total)
            && data.header.end == data.payload.start
    })
}

fn validate_tracks(media: &[TimedRange], total: u64) -> Result<(), TimelineError> {
    let mut previous = None;
    let mut count = 0;
    for track in media.chunk_by(|left, right| left.track == right.track) {
        let id = track[0].track;
        count += 1;
        if count > 16 || previous.is_some_and(|previous| previous >= id) {
            return Err(TimelineError::Malformed);
        }
        validate_track(track, total)?;
        previous = Some(id);
    }
    Ok(())
}

fn validate_track(track: &[TimedRange], total: u64) -> Result<(), TimelineError> {
    for (index, sample) in track.iter().enumerate() {
        let time = sample.time;
        PresentationTime::new(i128::from(time.start), i128::from(time.end), time.timescale)?;
        if !within(sample.bytes, total) || time.timescale != track[0].time.timescale {
            return Err(TimelineError::Malformed);
        }
        if !valid_sync(track, index) || !valid_decode(track, index) {
            return Err(TimelineError::Malformed);
        }
    }
    Ok(())
}

fn valid_sync(track: &[TimedRange], index: usize) -> bool {
    let Some(sync) = track[index].sync_sample else {
        return true;
    };
    sync as usize <= index
        && track
            .get(sync as usize)
            .is_some_and(|sample| sample.sync_sample == Some(sync))
}

fn valid_decode(track: &[TimedRange], index: usize) -> bool {
    let Some(start) = track[index].decode_start else {
        return true;
    };
    if index == 0 {
        return start == 0;
    }
    let previous = track[index - 1];
    let duration = i128::from(previous.time.end) - i128::from(previous.time.start);
    previous
        .decode_start
        .zip(u64::try_from(duration).ok())
        .and_then(|(start, duration)| start.checked_add(duration))
        == Some(start)
}

fn within(range: ByteRange, total: u64) -> bool {
    range.start < range.end && range.end <= total
}
