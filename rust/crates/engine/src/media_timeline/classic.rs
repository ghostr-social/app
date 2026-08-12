use super::boxes::{child, children, Atom};
use super::{TimedRange, TimelineError};

pub(super) mod samples;
mod tables;
use samples::map_samples;
use tables::TrackTables;

pub(crate) fn parse(moov: &Atom<'_>) -> Result<Vec<TimedRange>, TimelineError> {
    let mut media = Vec::new();
    for trak in children(moov)?
        .into_iter()
        .filter(|atom| &atom.kind == b"trak")
    {
        let Some(tables) = track_tables(&trak)? else {
            continue;
        };
        media.extend(map_samples(tables)?);
    }
    Ok(media)
}

fn track_tables(trak: &Atom<'_>) -> Result<Option<TrackTables>, TimelineError> {
    let Some(mdia) = child(trak, b"mdia")? else {
        return Ok(None);
    };
    let Some(mdhd) = child(&mdia, b"mdhd")? else {
        return Ok(None);
    };
    let Some(minf) = child(&mdia, b"minf")? else {
        return Ok(None);
    };
    let Some(stbl) = child(&minf, b"stbl")? else {
        return Ok(None);
    };
    TrackTables::parse(&mdhd, &stbl).map(Some)
}
