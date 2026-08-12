use crate::playback::{PlaybackSession, PlaybackStatus};
use crate::PostId;

#[test]
fn media_time_authorization_is_monotonic_and_session_scoped() {
    let session = PlaybackSession::new(PostId::new("video-a"), 7);
    let next = PlaybackSession::new(PostId::new("video-b"), 8);
    let mut status = PlaybackStatus::default();
    status.activate(session.clone());

    assert_eq!(status.authorize_media_ms(&session, 8_000), Some(8_000));
    assert_eq!(status.authorize_media_ms(&session, 4_000), Some(8_000));
    assert_eq!(status.authorize_media_ms(&next, 12_000), None);

    status.activate(next.clone());
    assert_eq!(status.authorize_media_ms(&next, 3_000), Some(3_000));
}
