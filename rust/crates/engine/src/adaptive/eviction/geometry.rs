use crate::ByteRange;

#[derive(Clone, Copy)]
pub(super) struct Extent {
    pub(super) range: ByteRange,
    pub(super) protected: bool,
}

pub(super) fn partition(present: &[ByteRange], protected: &[ByteRange]) -> Vec<Extent> {
    let protected = crate::media_timeline::normalize(protected.to_vec());
    crate::media_timeline::normalize(present.to_vec())
        .into_iter()
        .flat_map(|range| split(range, &protected))
        .collect()
}

fn split(range: ByteRange, protected: &[ByteRange]) -> Vec<Extent> {
    let mut extents = Vec::new();
    let mut cursor = range.start;
    for guard in protected
        .iter()
        .copied()
        .filter(|guard| overlaps(*guard, range))
    {
        push(&mut extents, cursor, guard.start.min(range.end), false);
        let start = cursor.max(guard.start);
        let end = range.end.min(guard.end);
        push(&mut extents, start, end, true);
        cursor = cursor.max(end);
    }
    push(&mut extents, cursor, range.end, false);
    extents
}

fn push(extents: &mut Vec<Extent>, start: u64, end: u64, protected: bool) {
    if start < end {
        extents.push(Extent {
            range: ByteRange::new(start, end),
            protected,
        });
    }
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
