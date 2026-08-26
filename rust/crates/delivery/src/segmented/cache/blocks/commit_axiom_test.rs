use super::*;

#[derive(Clone, Copy)]
pub(in super::super) struct CompleteValidation<'a> {
    generation: u64,
    offset: u64,
    block_bytes: u64,
    object: &'a PreparedObject,
}

impl<'a> CompleteValidation<'a> {
    pub(in super::super) const fn new(
        generation: u64,
        offset: u64,
        block_bytes: u64,
        object: &'a PreparedObject,
    ) -> Self {
        Self {
            generation,
            offset,
            block_bytes,
            object,
        }
    }
}

pub(in super::super) fn accepts_complete(
    record: &FocusRecord,
    input: CompleteValidation<'_>,
) -> bool {
    if invalid(record, input.generation, input.block_bytes) {
        return false;
    }
    if input.offset == 0 {
        return input.object.body.len() as u64 == input.block_bytes && record.assembly_bytes == 0;
    }
    let total = input.offset.checked_add(input.block_bytes);
    record.assembly_bytes == input.object.body.len() as u64
        && Some(input.object.body.len() as u64) == total
        && record
            .staged
            .iter()
            .any(|known| known.matches_identity(input.object, input.offset))
}

fn invalid(record: &FocusRecord, generation: u64, block_bytes: u64) -> bool {
    record.generation != generation
        || record.snapshot.phase == SegmentedPhase::Ready
        || block_bytes == 0
        || block_bytes > record.reserved_bytes
}
