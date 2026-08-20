use axum::http::HeaderValue;
use ghostr_gateway::progressive::range_header::{resolve, ResolvedRange};

fn resolved(header: Option<&str>, total: u64) -> ResolvedRange {
    let value = header.map(|text| HeaderValue::from_str(text).expect("header"));
    resolve(value.as_ref(), total)
}

fn partial(start: u64, end: u64) -> ResolvedRange {
    ResolvedRange::Partial { start, end }
}

#[test]
fn resolves_range_headers_against_the_total_length() {
    let cases = [
        (None, 10, ResolvedRange::Full),
        (Some("bytes=2-5"), 10, partial(2, 6)),
        (Some("bytes=2-"), 10, partial(2, 10)),
        (Some("bytes=0-9"), 10, partial(0, 10)),
        (Some("bytes=0-99"), 10, partial(0, 10)),
        (Some("bytes=-3"), 10, partial(7, 10)),
        (Some("bytes=-99"), 10, partial(0, 10)),
        (Some("bytes=10-"), 10, ResolvedRange::Unsatisfiable),
        (Some("bytes=12-14"), 10, ResolvedRange::Unsatisfiable),
        (Some("bytes=-0"), 10, ResolvedRange::Unsatisfiable),
        (Some("bytes=5-2"), 10, ResolvedRange::Full),
        (Some("bytes=+2-5"), 10, ResolvedRange::Full),
        (Some("bytes=2-+5"), 10, ResolvedRange::Full),
        (Some("bytes=+2-"), 10, ResolvedRange::Full),
        (Some("bytes=-+2"), 10, ResolvedRange::Full),
        (Some("bytes=0-1,3-4"), 10, ResolvedRange::Full),
        (Some("items=0-4"), 10, ResolvedRange::Full),
        (Some("bytes=x-y"), 10, ResolvedRange::Full),
        (Some("bytes=2-y"), 10, ResolvedRange::Full),
        (Some("bytes=-"), 10, ResolvedRange::Full),
        (Some("bytes=25"), 10, ResolvedRange::Full),
        (Some("bytes=0-"), 0, ResolvedRange::Unsatisfiable),
    ];
    for (header, total, expected) in cases {
        assert_eq!(resolved(header, total), expected, "case {header:?}/{total}");
    }
}
