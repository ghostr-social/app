use core::time::Duration;
use ghostr_gateway::hls::sessions::{HlsSessionLimits, HlsSessions};

#[tokio::test]
async fn bounds_live_sessions_until_an_owner_releases_one() {
    let limits = HlsSessionLimits::new(1, Duration::from_secs(60), 8).expect("limits");
    let sessions = HlsSessions::new(limits);
    let first = sessions
        .acquire(vec!["https://media.example/first.m3u8".to_owned()])
        .await
        .expect("first session");

    let full = sessions
        .acquire(vec!["https://media.example/second.m3u8".to_owned()])
        .await;

    assert!(full.is_err());
    assert!(sessions.release(&first).await);
    assert!(sessions
        .acquire(vec!["https://media.example/second.m3u8".to_owned()])
        .await
        .is_ok());
}
