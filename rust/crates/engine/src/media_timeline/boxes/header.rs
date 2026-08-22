use super::TimelineError;

#[derive(Clone, Copy)]
pub(super) struct Header {
    pub(super) size: usize,
    pub(super) kind: [u8; 4],
    pub(super) header: usize,
    pub(super) extends_to_end: bool,
}

pub(super) fn header(bytes: &[u8]) -> Result<Option<Header>, TimelineError> {
    if bytes.len() < 8 {
        return Ok(None);
    }
    let size32 = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let kind = bytes[4..8].try_into().unwrap();
    let Some((size, header, extends_to_end)) = dimensions(size32, bytes) else {
        return Ok(None);
    };
    if size < header as u64 || size > usize::MAX as u64 {
        return Ok(None);
    }
    Ok(Some(Header {
        size: size as usize,
        kind,
        header,
        extends_to_end,
    }))
}

pub(super) fn is_metadata(kind: [u8; 4]) -> bool {
    matches!(&kind, b"ftyp" | b"moov" | b"sidx")
}

pub(super) fn absolute(start: u64, relative: usize) -> Result<u64, TimelineError> {
    start
        .checked_add(relative as u64)
        .ok_or(TimelineError::Malformed)
}

fn dimensions(size32: u32, bytes: &[u8]) -> Option<(u64, usize, bool)> {
    Some(match size32 {
        0 => (bytes.len() as u64, 8, true),
        1 if bytes.len() >= 16 => (
            u64::from_be_bytes(bytes[8..16].try_into().unwrap()),
            16,
            false,
        ),
        1 => return None,
        value => (u64::from(value), 8, false),
    })
}
