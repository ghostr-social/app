use rust_lib_ghostr::video::video::{
    ffi_acquire_hls_playback, ffi_release_hls_playback, ffi_start_server,
};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn issues_and_releases_a_loopback_hls_playback_session() {
    let directory = cache_directory();
    let endpoint = ffi_start_server(
        directory.to_string_lossy().to_string(),
        1,
        1024,
        String::new(),
    )
    .await
    .expect("gateway endpoint");

    let session = ffi_acquire_hls_playback(vec![
        "https://media.example/master.m3u8".to_owned(),
        "https://mirror.example/master.m3u8".to_owned(),
    ])
    .await
    .expect("HLS session");

    let playback = reqwest::Url::parse(&session.playback_url).expect("playback URL");
    assert_eq!(playback.host_str(), Some("127.0.0.1"));
    assert_eq!(
        playback.port(),
        endpoint.split(':').nth(1).and_then(|raw| raw.parse().ok())
    );
    assert!(playback.path().ends_with("/index.m3u8"));
    assert!(ffi_release_hls_playback(session.session_id.clone()).await);
    assert!(!ffi_release_hls_playback(session.session_id).await);
    assert!(!ffi_release_hls_playback("not-a-session".to_owned()).await);
    fs::remove_dir_all(directory).expect("remove cache");
}

fn cache_directory() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ghostr-hls-gateway-{nonce}"))
}
