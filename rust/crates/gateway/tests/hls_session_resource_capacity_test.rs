use ghostr_gateway::hls_sessions::{HlsResourceId, HlsSessionLimits, HlsSessions};
use reqwest::Url;
use std::time::Duration;

#[tokio::test]
async fn long_and_rotating_playlists_do_not_accumulate_session_resources() {
    let limits = HlsSessionLimits::new(1, Duration::from_secs(60)).expect("limits");
    let sessions = HlsSessions::new(limits);
    let session = sessions
        .acquire(vec!["https://media.example/master.m3u8".to_owned()])
        .await
        .expect("session");
    let base = Url::parse("https://media.example/live/index.m3u8").expect("base URL");

    let first = sessions
        .rewrite_manifest(&session, &playlist(0), &base)
        .await
        .expect("first long playlist");
    let original_token = first_token(&first);
    let second = sessions
        .rewrite_manifest(&session, &playlist(4_000), &base)
        .await
        .expect("rotated long playlist");

    assert_ne!(original_token, first_token(&second));
    let token = HlsResourceId::parse(&original_token).expect("resource token");
    let resource = sessions.resource(&session, token).await;
    assert!(resource.is_some(), "old playlist capability remains usable");
}

fn playlist(start: usize) -> Vec<u8> {
    let mut value = String::from("#EXTM3U\n#EXT-X-TARGETDURATION:4\n");
    for index in start..start + 3_000 {
        value.push_str(&format!("#EXTINF:4,\n{index}.m4s\n"));
    }
    value.into_bytes()
}

fn first_token(manifest: &str) -> String {
    manifest
        .lines()
        .find_map(|line| {
            line.split_once("/assets/")
                .map(|(_, token)| token.to_owned())
        })
        .expect("asset capability")
}
