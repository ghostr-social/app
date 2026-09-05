use crate::media_timeline::TimelineError;
use crate::tests::media_timeline_dependency_support::{try_tail_timeline, with_track_atom};
use crate::tests::media_timeline_support::{atom, full_box, values};

#[test]
fn an_unimplemented_edit_list_cannot_claim_unedited_startup() {
    let edit = full_box(b"elst", values(&[1, 3_000, 1_000, 0x0001_0000]));
    let movie = with_track_atom(atom(b"edts", edit));

    assert_eq!(try_tail_timeline(&movie), Err(TimelineError::Unsupported));
}
