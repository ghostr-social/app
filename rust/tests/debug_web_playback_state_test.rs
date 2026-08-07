#![cfg(feature = "video-debug-web")]

#[test]
fn dashboard_reports_playing_only_after_media_really_starts() {
    let script = include_str!("../crates/gateway/src/debug_assets/player_events.js");

    assert!(script.contains("addEventListener(\"playing\""));
    assert!(!script.contains("addEventListener(\"play\""));
    assert!(script.contains("addEventListener(\"waiting\""));
    assert!(script.contains("addEventListener(\"stalled\""));
    assert!(script.contains("\"Buffering\""));
}
