use super::super::{FocusRecord, SegmentedPhase, StagedObject};
use crate::segmented::prepare::{PreparedComplete, PreparedObject};

pub(in crate::segmented::cache) fn commit_partial(
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

pub(in crate::segmented::cache) fn commit_prepared(
    record: &mut FocusRecord,
    prepared: PreparedComplete,
) {
    set_root(record, &prepared.object);
    let staged = StagedObject::complete_prepared(prepared);
    match find(record, staged.request_url()) {
        Some(index) => record.staged[index] = staged,
        None => record.staged.push(staged),
    }
    record.assembly_bytes = 0;
    finish(record);
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

#[cfg(test)]
#[path = "commit_axiom_test.rs"]
pub(crate) mod axiom_test_support;
