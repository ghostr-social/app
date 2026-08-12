use super::{MediaSegment, TimelineError};
use crate::ByteRange;

#[derive(Clone, Copy)]
pub(crate) struct Atom<'a> {
    pub(crate) kind: [u8; 4],
    pub(crate) start: u64,
    pub(crate) bytes: &'a [u8],
    header: usize,
}

impl<'a> Atom<'a> {
    pub(crate) fn payload(&self) -> &'a [u8] {
        &self.bytes[self.header..]
    }

    pub(crate) fn range(&self) -> Result<ByteRange, TimelineError> {
        let end = self
            .start
            .checked_add(self.bytes.len() as u64)
            .ok_or(TimelineError::Malformed)?;
        Ok(ByteRange::new(self.start, end))
    }
}

pub(crate) struct Scan<'a> {
    pub(crate) atoms: Vec<Atom<'a>>,
    pub(crate) truncated: bool,
}

pub(crate) fn scan<'a>(segments: &[MediaSegment<'a>]) -> Result<Scan<'a>, TimelineError> {
    let mut atoms = Vec::new();
    let mut truncated = false;
    for segment in segments {
        let found = scan_segment(*segment)?;
        truncated |= found.truncated;
        atoms.extend(found.atoms);
    }
    Ok(Scan { atoms, truncated })
}

fn scan_segment(segment: MediaSegment<'_>) -> Result<Scan<'_>, TimelineError> {
    let mut atoms = Vec::new();
    let mut cursor: usize = 0;
    let mut truncated = false;
    while cursor.saturating_add(8) <= segment.bytes.len() {
        let Some(header) = header(&segment.bytes[cursor..])? else {
            cursor += 1;
            continue;
        };
        if !is_metadata(header.kind) {
            cursor += 1;
            continue;
        }
        let end = cursor
            .checked_add(header.size)
            .ok_or(TimelineError::Malformed)?;
        if end > segment.bytes.len() {
            truncated = true;
            cursor += 1;
            continue;
        }
        atoms.push(Atom {
            kind: header.kind,
            start: absolute(segment.start, cursor)?,
            bytes: &segment.bytes[cursor..end],
            header: header.header,
        });
        cursor = end;
    }
    Ok(Scan { atoms, truncated })
}

pub(crate) fn children<'a>(parent: &Atom<'a>) -> Result<Vec<Atom<'a>>, TimelineError> {
    let mut children = Vec::new();
    let mut cursor = 0;
    let payload = parent.payload();
    while cursor < payload.len() {
        let header = header(&payload[cursor..])?.ok_or(TimelineError::Truncated)?;
        let end = cursor
            .checked_add(header.size)
            .ok_or(TimelineError::Malformed)?;
        if end > payload.len() {
            return Err(TimelineError::Truncated);
        }
        children.push(Atom {
            kind: header.kind,
            start: absolute(parent.start, parent.header + cursor)?,
            bytes: &payload[cursor..end],
            header: header.header,
        });
        cursor = end;
    }
    Ok(children)
}

pub(crate) fn child<'a>(
    parent: &Atom<'a>,
    kind: &[u8; 4],
) -> Result<Option<Atom<'a>>, TimelineError> {
    Ok(children(parent)?
        .into_iter()
        .find(|atom| &atom.kind == kind))
}

struct Header {
    size: usize,
    kind: [u8; 4],
    header: usize,
}

fn header(bytes: &[u8]) -> Result<Option<Header>, TimelineError> {
    if bytes.len() < 8 {
        return Ok(None);
    }
    let size32 = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let kind = bytes[4..8].try_into().unwrap();
    let Some((size, header)) = dimensions(size32, bytes) else {
        return Ok(None);
    };
    if size < header as u64 || size > usize::MAX as u64 {
        return Ok(None);
    }
    Ok(Some(Header {
        size: size as usize,
        kind,
        header,
    }))
}

fn dimensions(size32: u32, bytes: &[u8]) -> Option<(u64, usize)> {
    Some(match size32 {
        0 => (bytes.len() as u64, 8),
        1 if bytes.len() >= 16 => (u64::from_be_bytes(bytes[8..16].try_into().unwrap()), 16),
        1 => return None,
        value => (u64::from(value), 8),
    })
}

fn is_metadata(kind: [u8; 4]) -> bool {
    matches!(&kind, b"moov" | b"sidx")
}

fn absolute(start: u64, relative: usize) -> Result<u64, TimelineError> {
    start
        .checked_add(relative as u64)
        .ok_or(TimelineError::Malformed)
}
