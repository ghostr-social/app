use crate::ByteRange;

#[test]
fn length_is_the_half_open_span() {
    let cases = [(0, 0, 0), (0, 10, 10), (5, 8, 3)];

    for (start, end, expected) in cases {
        assert_eq!(ByteRange::new(start, end).len(), expected, "{start}..{end}");
    }
}

#[test]
fn empty_only_when_the_span_has_no_bytes() {
    assert!(ByteRange::new(4, 4).is_empty());
    assert!(!ByteRange::new(4, 5).is_empty());
}

#[test]
fn contains_offsets_inside_the_half_open_span() {
    let range = ByteRange::new(10, 20);
    let cases = [(9, false), (10, true), (19, true), (20, false)];

    for (offset, expected) in cases {
        assert_eq!(range.contains_offset(offset), expected, "offset {offset}");
    }
}
