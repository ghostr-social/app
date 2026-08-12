use crate::playback::{PlaybackSession, PlaybackStatus};
use crate::PostId;

#[test]
fn byte_authorization_never_moves_backward_within_a_session() {
    let session = PlaybackSession::new(PostId::new("video-a"), 7);
    let next = PlaybackSession::new(PostId::new("video-b"), 8);
    let mut status = PlaybackStatus::default();
    status.activate(session.clone());

    assert_eq!(status.authorize_bytes(&session, 8_000), Some(8_000));
    assert_eq!(status.authorize_bytes(&session, 4_000), Some(8_000));
    assert_eq!(status.authorize_bytes(&next, 12_000), None);

    status.activate(next.clone());
    assert_eq!(status.authorize_bytes(&next, 3_000), Some(3_000));
}
