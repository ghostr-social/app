//! Pure MP4 sniffing over the first bytes of a file: walks top-level boxes
//! only, so the engine can mark a post startable (moov in the head) or in
//! need of a tail probe (moov at the end).

/// True when a top-level `moov` box header is visible within the available
/// head bytes. False means the walk reached bytes we do not have yet (or a
/// corrupt header), so the moov — if any — lives beyond the head.
pub fn head_contains_moov(head: &[u8]) -> bool {
    let mut offset = 0usize;
    while head.len() >= 8 && offset <= head.len() - 8 {
        if &head[offset + 4..offset + 8] == b"moov" {
            return true;
        }
        match next_offset(head, offset) {
            Some(next) if next > offset => offset = next,
            _ => return false,
        }
    }
    false
}

/// The offset just past the box starting at `offset`: 32-bit size, 64-bit
/// largesize when the size field is 1, or none when the box runs to the end
/// of the file (size 0) or its header is corrupt.
fn next_offset(head: &[u8], offset: usize) -> Option<usize> {
    let declared = u32::from_be_bytes(head[offset..offset + 4].try_into().ok()?) as usize;
    let size = match declared {
        0 => return None,
        1 => largesize(head, offset)?,
        below if below < 8 => return None,
        _ => declared,
    };
    offset.checked_add(size)
}

fn largesize(head: &[u8], offset: usize) -> Option<usize> {
    let bytes = head.get(offset + 8..offset + 16)?;
    let size = u64::from_be_bytes(bytes.try_into().ok()?);
    usize::try_from(size).ok().filter(|size| *size >= 16)
}
