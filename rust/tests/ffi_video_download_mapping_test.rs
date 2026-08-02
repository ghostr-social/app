mod support;

use rust_lib_ghostr::video::ffi_models::ffi_video_download;
use std::path::PathBuf;
use support::fixtures::native_download;

#[test]
fn maps_pending_and_available_native_download_states() {
    let mut native = native_download("https://media.example/video.mp4");

    let pending = ffi_video_download(&native);
    assert!(pending.local_path.is_none());
    assert_eq!(pending.event.event_id, "event-id");
    assert_eq!(pending.nostr.user.name.as_deref(), Some("Ghost"));

    native.downloading = false;
    native.local_path = Some(PathBuf::from("/cache/video.mp4"));
    let available = ffi_video_download(&native);

    assert_eq!(available.id, native.id);
    assert_eq!(available.url, native.url);
    assert_eq!(available.title.as_deref(), Some("Relay clip"));
    assert_eq!(available.local_path.as_deref(), Some("/cache/video.mp4"));
    assert_eq!(available.nostr.song_name, "Original sound");
    assert_eq!(available.nostr.likes, "12");
    assert_eq!(available.nostr.comments, "4");
}
