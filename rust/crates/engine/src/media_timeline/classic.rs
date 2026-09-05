use super::boxes::{children, Atom};
use super::limits::ParserBudget;
use super::{TimedRange, TimelineError};

pub(super) mod samples;
mod tables;
use samples::map_samples;
use tables::TrackTables;

pub(super) fn parse(
    moov: &Atom<'_>,
    budget: &mut ParserBudget<'_>,
    media: &mut Vec<TimedRange>,
) -> Result<bool, TimelineError> {
    let mut video = false;
    for (track_id, trak) in children(moov, budget, 2)?
        .into_iter()
        .filter(|atom| &atom.kind == b"trak")
        .enumerate()
    {
        budget.track()?;
        let Some(track) = track_tables(&trak, budget)? else {
            continue;
        };
        video |= track.video;
        map_samples(&track.tables, budget, media, track_id as u16)?;
    }
    Ok(video)
}

struct TrackEvidence {
    tables: TrackTables,
    video: bool,
}

fn track_tables(
    trak: &Atom<'_>,
    budget: &mut ParserBudget<'_>,
) -> Result<Option<TrackEvidence>, TimelineError> {
    let trak_children = children(trak, budget, 3)?;
    if find(&trak_children, b"edts").is_some() {
        return Err(TimelineError::Unsupported);
    }
    let Some(mdia) = find(&trak_children, b"mdia") else {
        return Ok(None);
    };
    let mdia_children = children(&mdia, budget, 4)?;
    let Some(mdhd) = find(&mdia_children, b"mdhd") else {
        return Ok(None);
    };
    let Some(minf) = find(&mdia_children, b"minf") else {
        return Ok(None);
    };
    let minf_children = children(&minf, budget, 5)?;
    let Some(stbl) = find(&minf_children, b"stbl") else {
        return Ok(None);
    };
    let stbl_children = children(&stbl, budget, 6)?;
    let tables = TrackTables::parse(&mdhd, &stbl_children, budget)?;
    Ok(Some(TrackEvidence {
        tables,
        video: find(&mdia_children, b"hdlr").is_some_and(handler_is_video),
    }))
}

fn handler_is_video(atom: Atom<'_>) -> bool {
    atom.payload().get(8..12) == Some(b"vide")
}

fn find<'a>(atoms: &[Atom<'a>], kind: &[u8; 4]) -> Option<Atom<'a>> {
    atoms.iter().copied().find(|atom| &atom.kind == kind)
}
