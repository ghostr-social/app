use crate::media_timeline::{parse_mp4_segments, MediaSegment, TimelineError};
use crate::tests::media_timeline_support::{
    advanced_moov, atom, classic_from_tables, classic_moov, full_box, joined, values,
};

#[test]
fn parser_rejects_input_and_box_work_above_global_limits() {
    let oversized = vec![0_u8; 8 * 1024 * 1024 + 1];
    let boxes = (0..4_097)
        .map(|_| atom(b"free", Vec::new()))
        .collect::<Vec<_>>();
    let crowded = atom(b"moov", joined(&boxes));

    assert_eq!(parse(&oversized), Err(TimelineError::ResourceLimit));
    assert_eq!(parse(&crowded), Err(TimelineError::ResourceLimit));
}

#[test]
fn empty_segment_fanout_has_a_finite_parser_bound() {
    let segments = vec![MediaSegment::new(0, &[]); 10_000];

    assert_eq!(
        parse_mp4_segments(&segments),
        Err(TimelineError::ResourceLimit)
    );
}

#[test]
fn parser_rejects_track_and_global_sample_amplification() {
    let tracks = (0..17)
        .map(|_| classic_moov(&[1_000], &[1_000])[8..].to_vec())
        .collect::<Vec<_>>();
    let crowded = atom(b"moov", joined(&tracks));
    let first = advanced_moov(1_000, 100_001, 1);
    let second = advanced_moov(200_000, 100_001, 1);
    let samples = atom(
        b"moov",
        joined(&[first[8..].to_vec(), second[8..].to_vec()]),
    );

    assert_eq!(parse(&crowded), Err(TimelineError::ResourceLimit));
    assert_eq!(parse(&samples), Err(TimelineError::ResourceLimit));
}

#[test]
fn table_counts_are_bounded_before_capacity_is_reserved() {
    let moov = classic_from_tables(
        values(&[1, 1, 1_000]),
        values(&[1_000_000]),
        values(&[1, 1]),
        full_box(b"stco", values(&[1, 1_000])),
    );

    assert_eq!(parse(&moov), Err(TimelineError::ResourceLimit));
}

#[test]
fn aggregate_parser_allocations_have_a_hard_limit() {
    let moov = allocation_heavy_moov(200_000);

    assert_eq!(parse(&moov), Err(TimelineError::ResourceLimit));
}

fn allocation_heavy_moov(samples: u32) -> Vec<u8> {
    let mut chunks = Vec::with_capacity(samples as usize * 3 + 1);
    chunks.push(samples);
    for chunk in 1..=samples {
        chunks.extend([chunk, 1, 1]);
    }
    let mut offsets = Vec::with_capacity(samples as usize + 1);
    offsets.push(samples);
    offsets.extend(0..samples);
    classic_from_tables(
        values(&[1, samples, 1_000]),
        values(&chunks),
        values(&[1, samples]),
        full_box(b"stco", values(&offsets)),
    )
}

fn parse(bytes: &[u8]) -> Result<crate::media_timeline::MediaTimeline, TimelineError> {
    parse_mp4_segments(&[MediaSegment::new(0, bytes)])
}
