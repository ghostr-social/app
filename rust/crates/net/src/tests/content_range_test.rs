use crate::content_range;

#[test]
fn parses_the_exact_returned_span_and_total() {
    let parsed = content_range::parse("bytes 4-7/16").expect("content range");

    assert_eq!(parsed.range, 4..8);
    assert_eq!(parsed.total, Some(16));
}

#[test]
fn rejects_reversed_or_out_of_bounds_spans() {
    assert!(content_range::parse("bytes 8-7/16").is_none());
    assert!(content_range::parse("bytes 4-16/16").is_none());
    assert!(content_range::parse("bytes 4-7/not-a-total").is_none());
    assert!(content_range::parse("bytes +4-+7/+16").is_none());
}

#[test]
fn range_units_are_case_insensitive_and_unsatisfied_totals_are_typed() {
    let parsed = content_range::parse("BYTES 4-7/*").expect("case-insensitive range");

    assert_eq!(parsed.range, 4..8);
    assert_eq!(parsed.total, None);
    assert_eq!(content_range::parse_unsatisfied("bytes */16"), Some(16));
    assert_eq!(content_range::parse_unsatisfied("bytes */+16"), None);
    assert_eq!(content_range::parse_unsatisfied("bytes */*"), None);
    assert_eq!(content_range::parse_unsatisfied("bytes 0-3/16"), None);
}
