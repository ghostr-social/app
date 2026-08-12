use crate::media_timeline::{parse_mp4_segments, MediaSegment, TimelineError};
use crate::tests::media_timeline_support::{classic_from_tables, classic_moov, full_box, values};

#[test]
fn inconsistent_or_unbounded_sample_tables_are_rejected() {
    let scenarios = [
        custom(&[1, 2, 1_000], &[1, 1, 1, 1], &[0, 1, 100], &[1, 1_000]),
        custom(&[1, 1, 0], &[1, 1, 1, 1], &[0, 1, 100], &[1, 1_000]),
        custom(
            &[1, 1_000_001, 1_000],
            &[1, 1, 1, 1],
            &[0, 1, 100],
            &[1, 1_000],
        ),
        custom(&[1_000_001], &[1, 1, 1, 1], &[0, 1, 100], &[1, 1_000]),
        custom(&[1, 1, 1_000], &[1, 2, 1, 1], &[0, 1, 100], &[1, 1_000]),
        custom(&[1, 1, 1_000], &[1, 1, 0, 1], &[0, 1, 100], &[1, 1_000]),
        custom(&[1, 1, 1_000], &[1, 1, 1, 1], &[0, 1, 0], &[1, 1_000]),
    ];

    for moov in scenarios {
        assert_eq!(parse(&moov), Err(TimelineError::Malformed));
    }
}

#[test]
fn incomplete_chunk_coverage_and_unsorted_stsc_are_rejected() {
    let uncovered = custom(
        &[1, 2, 1_000],
        &[1, 1, 1, 1],
        &[0, 2, 100, 100],
        &[1, 1_000],
    );
    let unsorted = custom(
        &[1, 2, 1_000],
        &[2, 1, 1, 1, 1, 1, 1],
        &[0, 2, 100, 100],
        &[2, 1_000, 2_000],
    );

    assert_eq!(parse(&uncovered), Err(TimelineError::Malformed));
    assert_eq!(parse(&unsorted), Err(TimelineError::Malformed));
}

#[test]
fn unsupported_mdhd_and_truncated_co64_are_explicit() {
    let mut unsupported = classic_moov(&[1_000], &[100]);
    let mdhd = unsupported
        .windows(4)
        .position(|bytes| bytes == b"mdhd")
        .unwrap();
    unsupported[mdhd + 4] = 2;
    let mut co64 = full_box(b"co64", values(&[2]));
    co64.extend(1_000_u64.to_be_bytes());
    let truncated = classic_from_tables(
        values(&[1, 1, 1_000]),
        values(&[1, 1, 1, 1]),
        values(&[0, 1, 100]),
        co64,
    );

    assert_eq!(parse(&unsupported), Err(TimelineError::Unsupported));
    assert_eq!(parse(&truncated), Err(TimelineError::Truncated));
}

fn custom(stts: &[u32], stsc: &[u32], stsz: &[u32], stco: &[u32]) -> Vec<u8> {
    classic_from_tables(
        values(stts),
        values(stsc),
        values(stsz),
        full_box(b"stco", values(stco)),
    )
}

fn parse(bytes: &[u8]) -> Result<crate::media_timeline::MediaTimeline, TimelineError> {
    parse_mp4_segments(&[MediaSegment::new(0, bytes)])
}
