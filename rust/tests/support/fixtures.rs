use rust_lib_ghostr::video::native_models::{
    NativeEventIdentity, NativeUserData, NativeVideo, NativeVideoCacheKey, NativeVideoDelivery,
    NativeVideoDownload,
};
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn video_id() -> String {
    "a".repeat(64)
}

pub fn video_cache_key() -> NativeVideoCacheKey {
    NativeVideoCacheKey::UrlDerived(video_id())
}

pub fn video_cache_file_id() -> String {
    video_cache_key().storage_id().expect("cache file id")
}

pub fn native_video(url: &str) -> NativeVideo {
    NativeVideo {
        id: video_id(),
        expected_digest: None,
        fallback_urls: Vec::new(),
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

pub fn event_identity() -> NativeEventIdentity {
    NativeEventIdentity {
        event_id: "event-id".to_owned(),
        author_public_key_hex: "author-key".to_owned(),
        kind: 22,
        identifier: None,
        created_at: 42,
        content: "Relay clip".to_owned(),
        hashtags: Vec::new(),
    }
}

pub fn native_download(url: &str) -> NativeVideoDownload {
    NativeVideoDownload::new(video_id(), native_video(url), event_identity())
}

pub fn temp_directory(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

pub fn trusted_media_client() -> MediaHttpClient {
    MediaHttpClient::trusted().expect("trusted media client")
}
