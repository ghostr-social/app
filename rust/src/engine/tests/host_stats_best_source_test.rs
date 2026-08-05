//! `best_source` orders imeta URL candidates by expected host
//! performance so the downloader stops falling back blindly.

use crate::engine::host_stats::{host_of, HostStats};
use crate::engine::inventory_controller::Mode;
use std::time::Duration;

fn urls(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|url| (*url).to_owned()).collect()
}

#[test]
fn measured_fast_host_outranks_measured_slow_host() {
    let mut stats = HostStats::new();
    stats.record_transfer("slow.example", 100_000, Duration::from_secs(1));
    stats.record_transfer("fast.example", 5_000_000, Duration::from_secs(1));
    let candidates = urls(&["https://slow.example/v.mp4", "https://fast.example/v.mp4"]);

    let ordered = stats.best_source(&candidates, Mode::Comfort);

    let expected = urls(&["https://fast.example/v.mp4", "https://slow.example/v.mp4"]);
    assert_eq!(ordered, expected);
}

#[test]
fn tied_hosts_keep_their_imeta_order() {
    let stats = HostStats::new();
    let candidates = urls(&["https://a.example/v.mp4", "https://b.example/v.mp4"]);

    assert_eq!(stats.best_source(&candidates, Mode::Hunger), candidates);
}

#[test]
fn hunger_drops_a_failing_host_behind_an_unknown_mirror() {
    let mut stats = HostStats::new();
    stats.record_failure("flaky.example");
    let candidates = urls(&[
        "https://flaky.example/v.mp4",
        "https://mirror.example/v.mp4",
    ]);

    let ordered = stats.best_source(&candidates, Mode::Hunger);

    let expected = urls(&[
        "https://mirror.example/v.mp4",
        "https://flaky.example/v.mp4",
    ]);
    assert_eq!(ordered, expected);
}

#[test]
fn a_failing_host_sinks_behind_a_healthy_mirror_in_every_mode() {
    let mut stats = HostStats::new();
    stats.record_transfer("flaky.example", 8_000_000, Duration::from_secs(1));
    stats.record_failure("flaky.example");
    stats.record_transfer("steady.example", 300_000, Duration::from_secs(1));
    stats.record_success("steady.example");
    let candidates = urls(&[
        "https://flaky.example/v.mp4",
        "https://steady.example/v.mp4",
    ]);

    for mode in [Mode::Comfort, Mode::Hunger] {
        let ordered = stats.best_source(&candidates, mode);
        assert_eq!(ordered[0], "https://steady.example/v.mp4", "{mode:?}");
    }
}

#[test]
fn unparseable_urls_sink_to_the_end() {
    let stats = HostStats::new();
    let candidates = urls(&["not-a-url", "https://mirror.example/v.mp4"]);

    let ordered = stats.best_source(&candidates, Mode::Comfort);

    assert_eq!(ordered[0], "https://mirror.example/v.mp4");
}

#[test]
fn host_of_extracts_the_lowercased_authority() {
    assert_eq!(
        host_of("https://CDN.Example:8443/v.mp4?x=1"),
        Some("cdn.example:8443".to_owned())
    );
    assert_eq!(
        host_of("https://user@cdn.example/v.mp4"),
        Some("cdn.example".to_owned())
    );
    assert_eq!(host_of("no-scheme/v.mp4"), None);
    assert_eq!(host_of("https:///v.mp4"), None);
}
