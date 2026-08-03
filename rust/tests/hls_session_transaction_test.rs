use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use reqwest::Url;
use rust_lib_ghostr::video::hls_sessions::{
    HlsResourceId, HlsSessionId, HlsSessionLimits, HlsSessions,
};
use std::time::Duration;

#[tokio::test]
async fn failed_rewrites_leave_no_partial_resource_state() {
    let sessions = sessions();
    let id = acquire(&sessions).await;
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    let invalid = b"#EXTM3U\nfirst.m4s\nsecond.m4s\n#EXT-X-UNKNOWN:VALUE=1\n";

    assert!(sessions
        .rewrite_manifest(&id, invalid, &base)
        .await
        .is_err());
    let valid = sessions
        .rewrite_manifest(&id, b"#EXTM3U\nthird.m4s\nfourth.m4s\n", &base)
        .await
        .expect("valid rewrite after failure");
    assert_eq!(valid.matches("/assets/").count(), 2);
}

#[tokio::test]
async fn resource_capabilities_are_authenticated() {
    let sessions = sessions();
    let id = acquire(&sessions).await;
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    let manifest = sessions
        .rewrite_manifest(&id, b"#EXTM3U\nsegment.m4s\n", &base)
        .await
        .expect("manifest");
    let token = manifest.split("/assets/").nth(1).expect("token").trim();
    assert!(token.len() > 40);
    let original = HlsResourceId::parse(token).expect("resource token");
    assert!(sessions.resource(&id, original).await.is_some());
    let mut tampered = token.as_bytes().to_vec();
    tampered[0] = if tampered[0] == b'A' { b'B' } else { b'A' };
    let tampered = String::from_utf8(tampered).expect("ASCII token");
    let token = HlsResourceId::parse(&tampered).expect("well-formed token");
    assert!(sessions.resource(&id, token).await.is_none());
}

#[tokio::test]
async fn rejects_structurally_invalid_resource_capabilities() {
    let sessions = sessions();
    let id = acquire(&sessions).await;

    for payload in [vec![], vec![2, 0, b'a'], vec![1, 9, b'a']] {
        let mut raw = payload;
        raw.extend([0_u8; 32]);
        let encoded = URL_SAFE_NO_PAD.encode(raw);
        let token = HlsResourceId::parse(&encoded).expect("resource token");
        assert!(sessions.resource(&id, token).await.is_none());
    }
}

fn sessions() -> HlsSessions {
    let limits = HlsSessionLimits::new(1, Duration::from_secs(60)).expect("limits");
    HlsSessions::new(limits)
}

async fn acquire(sessions: &HlsSessions) -> HlsSessionId {
    sessions
        .acquire(vec!["https://media.example/live.m3u8".to_owned()])
        .await
        .expect("session")
}
