mod support;

use rust_lib_ghostr::video::ffi_models::ffi_video_download;
use std::path::PathBuf;
use support::fixtures::native_download;

#[test]
fn maps_pending_and_available_native_download_states() {
    let mut native = native_download("https://media.example/video.mp4");
    native.nostr.expected_digest = Some("b".repeat(64));
    native.nostr.fallback_urls = vec!["https://mirror.example/video.mp4".to_owned()];

    let pending = ffi_video_download(&native);
    assert!(pending.local_path.is_none());
    assert_eq!(pending.event.event_id, "event-id");
    assert_eq!(pending.event.author_public_key_hex, "author-key");
    assert_eq!(pending.event.kind, 22);
    assert!(pending.event.identifier.is_none());
    assert_eq!(pending.event.created_at, 42);
    assert_eq!(pending.event.content, "Relay clip");
    assert_eq!(pending.nostr.user.name.as_deref(), Some("Ghost"));
    assert_eq!(pending.nostr.expected_digest, native.nostr.expected_digest);
    assert_eq!(pending.nostr.fallback_urls, native.nostr.fallback_urls);

    native.mark_available(PathBuf::from("/cache/video.mp4"));
    let available = ffi_video_download(&native);

    assert_eq!(available.id, native.id);
    assert_eq!(available.url, native.url);
    assert_eq!(available.title.as_deref(), Some("Relay clip"));
    assert_eq!(available.local_path.as_deref(), Some("/cache/video.mp4"));
    assert_eq!(available.nostr.song_name, "Original sound");
    assert_eq!(available.nostr.likes, "12");
    assert_eq!(available.nostr.comments, "4");
}
