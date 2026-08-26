use super::TimelineError;

#[derive(Clone, Copy)]
pub(super) struct Header {
    pub(super) size: usize,
    pub(super) kind: [u8; 4],
    pub(super) header: usize,
    pub(super) extends_to_end: bool,
}

pub(super) fn header(bytes: &[u8]) -> Option<Header> {
    if bytes.len() < 8 {
        return None;
    }
    let size32 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let kind = [bytes[4], bytes[5], bytes[6], bytes[7]];
    let (size, header, extends_to_end) = dimensions(size32, bytes)?;
    if size < header as u64 || size > usize::MAX as u64 {
        return None;
    }
    Some(Header {
        size: size as usize,
        kind,
        header,
        extends_to_end,
    })
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
            u64::from_be_bytes([
                bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ]),
            16,
            false,
        ),
        1 => return None,
        value => (u64::from(value), 8, false),
    })
}
