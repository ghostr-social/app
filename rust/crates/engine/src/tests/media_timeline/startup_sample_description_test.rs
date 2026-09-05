use crate::media_timeline::TimelineError;
use crate::tests::media_timeline_dependency_support::try_tail_timeline;
use crate::tests::media_timeline_support::classic_moov;

#[test]
fn a_nondefault_sample_description_requires_a_supported_selection_plan() {
    let mut movie = classic_moov(&[100, 200], &[100; 2]);
    let at = movie
        .windows(4)
        .position(|bytes| bytes == b"stsc")
        .expect("fixture");
    movie[at + 20..at + 24].copy_from_slice(&2_u32.to_be_bytes());

    assert_eq!(try_tail_timeline(&movie), Err(TimelineError::Unsupported));
}
