use super::boxes::Atom;
use super::limits::ParserBudget;
use super::{classic, sidx, TimedRange, TimelineError};
use crate::ByteRange;

pub(super) struct SelectedMedia {
    pub(super) ranges: Vec<TimedRange>,
    pub(super) classic_movie: Option<ByteRange>,
    pub(super) classic_movie_top_level: bool,
    pub(super) classic_video: bool,
}

pub(super) fn parse(
    atoms: &[Atom<'_>],
    truncated: bool,
    budget: &mut ParserBudget<'_>,
) -> Result<SelectedMedia, TimelineError> {
    let parsed = parse_atoms(atoms, budget)?;
    select_media(parsed, truncated)
}

struct ParsedMedia {
    classic: Vec<ClassicMapping>,
    fragmented: Vec<TimedRange>,
}

struct ClassicMapping {
    movie: ByteRange,
    movie_top_level: bool,
    ranges: Vec<TimedRange>,
    video: bool,
}

fn parse_atoms(
    atoms: &[Atom<'_>],
    budget: &mut ParserBudget<'_>,
) -> Result<ParsedMedia, TimelineError> {
    let mut parsed = ParsedMedia {
        classic: Vec::new(),
        fragmented: Vec::new(),
    };
    for atom in atoms {
        budget.work(1)?;
        parse_atom(*atom, budget, &mut parsed)?;
    }
    Ok(parsed)
}

fn parse_atom(
    atom: Atom<'_>,
    budget: &mut ParserBudget<'_>,
    parsed: &mut ParsedMedia,
) -> Result<(), TimelineError> {
    match &atom.kind {
        b"moov" => parse_movie(atom, budget, &mut parsed.classic),
        b"sidx" => sidx::parse(&atom, budget, &mut parsed.fragmented),
        b"ftyp" => Ok(()),
        _ => Err(TimelineError::Unsupported),
    }
}

fn parse_movie(
    atom: Atom<'_>,
    budget: &mut ParserBudget<'_>,
    mappings: &mut Vec<ClassicMapping>,
) -> Result<(), TimelineError> {
    let mut ranges = Vec::new();
    let video = classic::parse(&atom, budget, &mut ranges)?;
    if !ranges.is_empty() {
        mappings.push(ClassicMapping {
            movie: atom.range()?,
            movie_top_level: atom.is_top_level(),
            ranges,
            video,
        });
    }
    Ok(())
}

fn select_media(parsed: ParsedMedia, truncated: bool) -> Result<SelectedMedia, TimelineError> {
    let selected = select_classic(parsed.classic)
        .map(classic_selection)
        .unwrap_or_else(|| fragmented_selection(parsed.fragmented));
    if selected.ranges.is_empty() {
        return Err(if truncated {
            TimelineError::Truncated
        } else {
            TimelineError::Unavailable
        });
    }
    Ok(selected)
}

fn classic_selection(classic: ClassicMapping) -> SelectedMedia {
    SelectedMedia {
        ranges: classic.ranges,
        classic_movie: Some(classic.movie),
        classic_movie_top_level: classic.movie_top_level,
        classic_video: classic.video,
    }
}

fn fragmented_selection(ranges: Vec<TimedRange>) -> SelectedMedia {
    SelectedMedia {
        ranges,
        classic_movie: None,
        classic_movie_top_level: false,
        classic_video: false,
    }
}

fn select_classic(mut mappings: Vec<ClassicMapping>) -> Option<ClassicMapping> {
    let selected = mappings
        .iter()
        .position(|mapping| mapping.video && mapping.movie_top_level)
        .or_else(|| mappings.iter().position(|mapping| mapping.video))
        .unwrap_or(0);
    (!mappings.is_empty()).then(|| mappings.swap_remove(selected))
}
