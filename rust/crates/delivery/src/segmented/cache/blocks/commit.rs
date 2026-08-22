use super::super::{FocusRecord, SegmentedPhase, StagedObject};
use crate::segmented::prepare::PreparedObject;

pub(super) struct CompleteValidation<'a> {
    generation: u64,
    offset: u64,
    block_bytes: u64,
    object: &'a PreparedObject,
}

impl<'a> CompleteValidation<'a> {
    pub(super) const fn new(
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

pub(super) fn accepts_complete(record: &FocusRecord, input: CompleteValidation<'_>) -> bool {
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

pub(super) fn commit_partial(
    record: &mut FocusRecord,
    offset: u64,
    object: PreparedObject,
) -> Option<()> {
    set_root(record, &object);
    let found = find(record, &object.request_url);
    match (offset, found) {
        (0, Some(index)) => record.staged[index] = StagedObject::partial(object),
        (0, None) => record.staged.push(StagedObject::partial(object)),
        (_, Some(index)) => record.staged[index].push(object, offset)?,
        _ => return None,
    }
    finish(record);
    Some(())
}

pub(super) fn commit_complete(record: &mut FocusRecord, offset: u64, object: PreparedObject) {
    set_root(record, &object);
    match (offset, find(record, &object.request_url)) {
        (0, Some(index)) | (_, Some(index)) => {
            record.staged[index] = StagedObject::complete(object);
        }
        (_, None) => record.staged.push(StagedObject::complete(object)),
    }
    record.assembly_bytes = 0;
    finish(record);
}

fn invalid(record: &FocusRecord, generation: u64, block_bytes: u64) -> bool {
    record.generation != generation
        || record.snapshot.phase == SegmentedPhase::Ready
        || block_bytes == 0
        || block_bytes > record.reserved_bytes
}

fn set_root(record: &mut FocusRecord, object: &PreparedObject) {
    if record.staged.is_empty() {
        record.root_source = Some(object.request_url.clone());
    }
}

fn finish(record: &mut FocusRecord) {
    record.reserved_bytes = 0;
    record.snapshot.bytes_present = record.staged.iter().map(StagedObject::len).sum();
    record.snapshot.phase = SegmentedPhase::Queued;
    record.snapshot.eta_ms = None;
    record.snapshot.detail = None;
}

fn find(record: &FocusRecord, url: &str) -> Option<usize> {
    record
        .staged
        .iter()
        .position(|known| known.request_url() == url)
}
