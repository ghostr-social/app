use rust_lib_ghostr::video::event_identity::CanonicalNativeVideo;
use rust_lib_ghostr::video::ffi_models::FfiNostrEventIdentity;
use rust_lib_ghostr::video::native_models::{
    NativeUserData, NativeVideo, NativeVideoDelivery, NativeVideoDownload,
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn video_id() -> String {
    "a".repeat(64)
}

pub fn native_video(url: &str) -> NativeVideo {
    NativeVideo {
        id: video_id(),
        user: NativeUserData {
            npub: Some("npub1author".to_owned()),
            name: Some("Ghost".to_owned()),
            profile_picture: Some("https://media.example/avatar.png".to_owned()),
        },
        title: "Relay clip".to_owned(),
        song_name: "Original sound".to_owned(),
        comments: "4".to_owned(),
        likes: "12".to_owned(),
        url: url.to_owned(),
        delivery: NativeVideoDelivery::Progressive,
    }
}

pub fn event_identity() -> FfiNostrEventIdentity {
    FfiNostrEventIdentity {
        event_id: "event-id".to_owned(),
        author_public_key_hex: "author-key".to_owned(),
        kind: 22,
        identifier: None,
        created_at: 42,
        content: "Relay clip".to_owned(),
    }
}

pub fn native_download(url: &str) -> NativeVideoDownload {
    NativeVideoDownload::new(video_id(), native_video(url), event_identity())
}

pub fn canonical_video(url: &str) -> CanonicalNativeVideo {
    CanonicalNativeVideo {
        inventory_id: video_id(),
        coordinate: "event-id".to_owned(),
        identity: event_identity(),
        video: native_video(url),
    }
}

pub fn temp_directory(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}
