#![cfg(feature = "video-debug-web")]

mod support;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;

#[tokio::test]
async fn disabled_nostr_does_not_publish_an_empty_focus_over_the_demo() {
    let cache = support::fixtures::temp_directory("video-debug-disabled-nostr");
    let mut child = Command::new(env!("CARGO_BIN_EXE_video-debug"))
        .env("GHOSTR_VIDEO_DEBUG_CACHE", &cache)
        .env("GHOSTR_NOSTR_RELAYS", "")
        .env("GHOSTR_NOSTR_SEARCH_RELAYS", "")
        .stdout(Stdio::piped())
        .spawn()
        .expect("start video-debug");
    let mut line = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut line)
        .expect("dashboard URL");
    let url = line
        .split_whitespace()
        .last()
        .expect("URL")
        .trim_end_matches("/debug");

    tokio::time::sleep(Duration::from_millis(250)).await;
    let body = reqwest::get(format!("{url}/debug/api/state"))
        .await
        .expect("state request")
        .bytes()
        .await
        .expect("state body");
    let state: serde_json::Value = serde_json::from_slice(&body).expect("state JSON");

    child.kill().expect("stop video-debug");
    child.wait().expect("join video-debug");
    assert_eq!(state["nostr"]["stage"], "loading");
    assert_eq!(state["nostr"]["revision"], 0);
}
