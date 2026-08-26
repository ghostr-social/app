use super::limits::ParserBudget;
use super::TimelineError;
use crate::ByteRange;
use header::{absolute, header};

mod header;
mod scan;

pub(crate) use scan::scan;

#[derive(Clone, Copy)]
pub(crate) struct Atom<'a> {
    pub(super) kind: [u8; 4],
    pub(super) start: u64,
    bytes: &'a [u8],
    header: usize,
    top_level: bool,
}

impl<'a> Atom<'a> {
    pub(super) fn payload(&self) -> &'a [u8] {
        &self.bytes[self.header..]
    }

    pub(super) fn range(&self) -> Result<ByteRange, TimelineError> {
        let end = self
            .start
            .checked_add(self.bytes.len() as u64)
            .ok_or(TimelineError::Malformed)?;
        Ok(ByteRange::new(self.start, end))
    }

    pub(super) fn is_top_level(&self) -> bool {
        self.top_level
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MediaData {
    pub(super) header: ByteRange,
    pub(super) payload: ByteRange,
}

pub(crate) struct Scan<'a> {
    pub(super) atoms: Vec<Atom<'a>>,
    pub(super) media_data: Vec<MediaData>,
    pub(super) fragmented_markers: usize,
    pub(super) truncated: bool,
}

pub(crate) fn children<'a>(
    parent: &Atom<'a>,
    budget: &mut ParserBudget<'_>,
    depth: usize,
) -> Result<Vec<Atom<'a>>, TimelineError> {
    let mut children = Vec::new();
    let mut cursor = 0;
    let payload = parent.payload();
    while cursor < payload.len() {
        budget.work(1)?;
        let header = header(&payload[cursor..]).ok_or(TimelineError::Truncated)?;
        let end = cursor
            .checked_add(header.size)
            .ok_or(TimelineError::Malformed)?;
        if end > payload.len() {
            return Err(TimelineError::Truncated);
        }
        budget.box_at(header.size, depth)?;
        let child = Atom {
            kind: header.kind,
            start: absolute(parent.start, parent.header + cursor)?,
            bytes: &payload[cursor..end],
            header: header.header,
            top_level: false,
        };
        budget.push(&mut children, child)?;
        cursor = end;
    }
    Ok(children)
}
