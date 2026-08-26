use super::header::{absolute, header, is_metadata, Header};
use super::{Atom, MediaData, Scan};
use crate::media_timeline::limits::ParserBudget;
use crate::media_timeline::{MediaSegment, TimelineError};
use crate::ByteRange;

pub(crate) fn scan<'a>(
    segments: &[MediaSegment<'a>],
    budget: &mut ParserBudget<'_>,
) -> Result<Scan<'a>, TimelineError> {
    let mut scan = Scan {
        atoms: Vec::new(),
        media_data: Vec::new(),
        fragmented_markers: 0,
        truncated: false,
    };
    let mut boundary = Some(0);
    for segment in segments {
        budget.work(1)?;
        scan_segment(*segment, &mut boundary, budget, &mut scan)?;
    }
    Ok(scan)
}

fn scan_segment<'a>(
    segment: MediaSegment<'a>,
    boundary: &mut Option<u64>,
    budget: &mut ParserBudget<'_>,
    scan: &mut Scan<'a>,
) -> Result<(), TimelineError> {
    let end = absolute(segment.start, segment.bytes.len())?;
    match boundary.filter(|value| *value >= segment.start && *value < end) {
        Some(start) => scan_chain(segment, start, boundary, budget, scan),
        None => scan_search(segment, budget, scan),
    }
}

fn scan_chain<'a>(
    segment: MediaSegment<'a>,
    start: u64,
    boundary: &mut Option<u64>,
    budget: &mut ParserBudget<'_>,
    scan: &mut Scan<'a>,
) -> Result<(), TimelineError> {
    let mut cursor = usize::try_from(start - segment.start)
        .map_err(|_conversion_error| TimelineError::Malformed)?;
    while cursor.saturating_add(8) <= segment.bytes.len() {
        let parsed = header(&segment.bytes[cursor..]).ok_or(TimelineError::Malformed)?;
        if parsed.extends_to_end {
            scan_open_ended(segment, cursor, parsed, budget, scan)?;
            *boundary = None;
            return Ok(());
        }
        let box_start = absolute(segment.start, cursor)?;
        let box_end = box_start
            .checked_add(parsed.size as u64)
            .ok_or(TimelineError::Malformed)?;
        *boundary = Some(box_end);
        record_non_metadata(box_start, box_end, parsed, budget, scan)?;
        let end = cursor
            .checked_add(parsed.size)
            .ok_or(TimelineError::Malformed)?;
        if end > segment.bytes.len() {
            if is_metadata(parsed.kind) {
                scan.truncated = true;
                *boundary = None;
            }
            return Ok(());
        }
        if is_metadata(parsed.kind) {
            push_atom(segment, cursor, parsed, true, budget, &mut scan.atoms)?;
        }
        budget.scan(parsed.size)?;
        cursor = end;
    }
    Ok(())
}

fn record_non_metadata(
    start: u64,
    end: u64,
    parsed: Header,
    budget: &mut ParserBudget<'_>,
    scan: &mut Scan<'_>,
) -> Result<(), TimelineError> {
    if is_metadata(parsed.kind) {
        return Ok(());
    }
    budget.box_header(1)?;
    if matches!(&parsed.kind, b"moof" | b"mfra") {
        scan.fragmented_markers = scan.fragmented_markers.saturating_add(1);
        return Ok(());
    }
    if &parsed.kind != b"mdat" {
        return Ok(());
    }
    let payload_start = start
        .checked_add(parsed.header as u64)
        .ok_or(TimelineError::Malformed)?;
    budget.push(
        &mut scan.media_data,
        MediaData {
            header: ByteRange::new(start, payload_start),
            payload: ByteRange::new(payload_start, end),
        },
    )
}

fn scan_open_ended<'a>(
    segment: MediaSegment<'a>,
    cursor: usize,
    parsed: Header,
    budget: &mut ParserBudget<'_>,
    scan: &mut Scan<'a>,
) -> Result<(), TimelineError> {
    let remaining = segment.bytes.len() - cursor;
    if is_metadata(parsed.kind) {
        push_atom(segment, cursor, parsed, false, budget, &mut scan.atoms)?;
    } else {
        budget.box_header(1)?;
    }
    budget.scan(remaining)
}

fn scan_search<'a>(
    segment: MediaSegment<'a>,
    budget: &mut ParserBudget<'_>,
    scan: &mut Scan<'a>,
) -> Result<(), TimelineError> {
    let mut cursor: usize = 0;
    while cursor.saturating_add(8) <= segment.bytes.len() {
        let Some(parsed) = header(&segment.bytes[cursor..]) else {
            advance(&mut cursor, budget)?;
            continue;
        };
        let Some(end) = cursor.checked_add(parsed.size) else {
            return Err(TimelineError::Malformed);
        };
        if !is_metadata(parsed.kind) || end > segment.bytes.len() {
            scan.truncated |= is_metadata(parsed.kind) && end > segment.bytes.len();
            advance(&mut cursor, budget)?;
            continue;
        }
        push_atom(segment, cursor, parsed, false, budget, &mut scan.atoms)?;
        cursor = end;
        budget.work(1)?;
    }
    Ok(())
}

fn push_atom<'a>(
    segment: MediaSegment<'a>,
    cursor: usize,
    parsed: Header,
    top_level: bool,
    budget: &mut ParserBudget<'_>,
    atoms: &mut Vec<Atom<'a>>,
) -> Result<(), TimelineError> {
    let end = cursor + parsed.size;
    budget.box_at(parsed.size, 1)?;
    budget.push(
        atoms,
        Atom {
            kind: parsed.kind,
            start: absolute(segment.start, cursor)?,
            bytes: &segment.bytes[cursor..end],
            header: parsed.header,
            top_level,
        },
    )
}

fn advance(cursor: &mut usize, budget: &mut ParserBudget<'_>) -> Result<(), TimelineError> {
    *cursor += 1;
    budget.scan(1)
}
