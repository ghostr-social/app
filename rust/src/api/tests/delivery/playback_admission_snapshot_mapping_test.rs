use crate::api::playback_types::FfiPlaybackAdmissionSnapshot;
use ghostr_delivery::playback_admission::PlaybackAdmissionSnapshot;

#[test]
fn maps_typed_playback_admission_snapshot_for_flutter() {
    let mapped = FfiPlaybackAdmissionSnapshot::from(PlaybackAdmissionSnapshot::default());

    assert_eq!(mapped.accepted, 0);
    assert_eq!(mapped.inactive_delivery, 0);
    assert_eq!(mapped.stale_session, 0);
    assert_eq!(mapped.stale_sequence, 0);
    assert_eq!(mapped.last_accepted_delivery_id, None);
}
