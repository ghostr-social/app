use crate::api::delivery::playback_mapping::playback_presentation;
use crate::api::playback_types::FfiPlaybackPresentation;

#[test]
fn maps_only_ordered_presentations_with_exact_session_identity() {
    let mapped = playback_presentation(input(7, 9)).expect("test fixture precondition must hold");

    assert_eq!(mapped.session().post().as_str(), "video_1");
    assert_eq!(mapped.session().generation(), 7);
    assert_eq!(mapped.sequence(), 9);
    assert_eq!(mapped.observed_at_ms(), 321);
    assert!(playback_presentation(input(0, 9)).is_err());
    assert!(playback_presentation(input(7, 0)).is_err());
}

fn input(generation: u64, sequence: u64) -> FfiPlaybackPresentation {
    FfiPlaybackPresentation {
        post_id: "video_1".into(),
        generation,
        sequence,
        observed_at_ms: 321,
    }
}
