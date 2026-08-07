#![cfg(feature = "video-debug-web")]

mod support;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn standalone_debug_server_prints_its_page_url() {
    let cache = support::fixtures::temp_directory("video-debug-binary");
    let mut child = Command::new(env!("CARGO_BIN_EXE_video-debug"))
        .env("GHOSTR_VIDEO_DEBUG_CACHE", &cache)
        .env("GHOSTR_NOSTR_RELAYS", "")
        .env("GHOSTR_NOSTR_SEARCH_RELAYS", "")
        .stdout(Stdio::piped())
        .spawn()
        .expect("start video-debug");
    let stdout = child.stdout.take().expect("stdout");
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(stdout).read_line(&mut line);
        let _ = sender.send(line);
    });

    let line = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("dashboard URL");
    child.kill().expect("stop video-debug");
    child.wait().expect("join video-debug");

    assert!(line.starts_with("Video debug dashboard: http://127.0.0.1:"));
    assert!(line.trim_end().ends_with("/debug"));
}
