mod support;

use rust_lib_ghostr::video::video::{ffi_acquire_hls_playback, ffi_release_hls_playback};
use support::{engine, fixtures::temp_directory};

#[tokio::test]
async fn issues_and_releases_a_loopback_hls_playback_session() {
    assert!(!ffi_release_hls_playback("before-start".to_owned()).await);
    let directory = temp_directory("ghostr-hls-gateway");
    let endpoint = engine::start(&directory, 1024)
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
    std::fs::remove_dir_all(directory).expect("remove cache");
}
