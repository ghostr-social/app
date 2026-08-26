use core::time::Duration;
use ghostr_gateway::hls::sessions::{HlsSessionLimits, HlsSessions};
use reqwest::Url;

#[tokio::test(start_paused = true)]
async fn expired_sessions_cannot_be_used_and_release_capacity() {
    let ttl = Duration::from_secs(10);
    let limits = HlsSessionLimits::new(1, ttl, 8).expect("limits");
    let sessions = HlsSessions::new(limits);
    let id = sessions
        .acquire(vec!["https://media.example/live.m3u8".to_owned()])
        .await
        .expect("session");
    tokio::time::advance(ttl + Duration::from_millis(1)).await;

    assert!(sessions.sources(&id).await.is_none());
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    assert!(sessions
        .rewrite_manifest(&id, b"#EXTM3U\nsegment.m4s\n", &base)
        .await
        .is_err());
    assert!(sessions
        .acquire(vec!["https://media.example/next.m3u8".to_owned()])
        .await
        .is_ok());
}
