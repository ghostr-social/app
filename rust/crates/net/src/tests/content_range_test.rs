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
}
