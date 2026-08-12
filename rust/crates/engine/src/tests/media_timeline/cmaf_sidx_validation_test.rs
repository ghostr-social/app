use crate::media_timeline::{parse_mp4_segments, MediaSegment, TimelineError};
use crate::tests::cmaf_timeline_support::{cmaf_sidx, cmaf_sidx_v1};

#[test]
fn invalid_sidx_fields_are_rejected() {
    let mut zero_scale = cmaf_sidx(&[100], &[1_000], 0);
    zero_scale[16..20].fill(0);
    let mut hierarchical = cmaf_sidx(&[100], &[1_000], 0);
    hierarchical[32] |= 0x80;
    let zero_size = cmaf_sidx(&[0], &[1_000], 0);
    let zero_duration = cmaf_sidx(&[100], &[0], 0);

    assert_eq!(parse(&zero_scale), Err(TimelineError::Malformed));
    assert_eq!(parse(&hierarchical), Err(TimelineError::Unsupported));
    assert_eq!(parse(&zero_size), Err(TimelineError::Malformed));
    assert_eq!(parse(&zero_duration), Err(TimelineError::Malformed));
}

#[test]
fn version_one_sidx_uses_64_bit_time_and_offset_fields() {
    let sidx = cmaf_sidx_v1(1_000, 4_000, 100, 500);
    let timeline = parse(&sidx).unwrap();

    assert_eq!(timeline.duration_ms(), 4_500);
}

#[test]
fn unsupported_or_truncated_sidx_headers_are_explicit() {
    let mut unsupported = cmaf_sidx(&[100], &[1_000], 0);
    unsupported[8] = 2;
    let empty = 8_u32
        .to_be_bytes()
        .into_iter()
        .chain(*b"sidx")
        .collect::<Vec<_>>();
    let mut truncated_v1 = cmaf_sidx_v1(1_000, 0, 100, 500);
    truncated_v1.truncate(31);
    let truncated_size = (truncated_v1.len() as u32).to_be_bytes();
    truncated_v1[0..4].copy_from_slice(&truncated_size);

    assert_eq!(parse(&unsupported), Err(TimelineError::Unsupported));
    assert_eq!(parse(&empty), Err(TimelineError::Truncated));
    assert_eq!(parse(&truncated_v1), Err(TimelineError::Truncated));
}

fn parse(bytes: &[u8]) -> Result<crate::media_timeline::MediaTimeline, TimelineError> {
    parse_mp4_segments(&[MediaSegment::new(0, bytes)])
}
