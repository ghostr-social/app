use crate::ByteRange;

pub fn normalize(mut ranges: Vec<ByteRange>) -> Vec<ByteRange> {
    ranges.retain(|range| !range.is_empty());
    ranges.sort_by_key(|range| (range.start, range.end));
    let mut write = 0;
    for read in 0..ranges.len() {
        write = merge_at(&mut ranges, write, read);
    }
    ranges.truncate(write);
    ranges
}

fn merge_at(ranges: &mut [ByteRange], write: usize, read: usize) -> usize {
    let next = ranges[read];
    if write > 0 && next.start <= ranges[write - 1].end {
        ranges[write - 1].end = ranges[write - 1].end.max(next.end);
        return write;
    }
    ranges[write] = next;
    write + 1
}
