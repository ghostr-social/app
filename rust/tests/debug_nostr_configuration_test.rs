#![cfg(feature = "video-debug-web")]

use rust_lib_ghostr::api::debug::nostr::DebugNostrConfiguration;

#[test]
fn standalone_debugger_has_real_nostr_relays_by_default() {
    let configuration = DebugNostrConfiguration::default();

    assert_eq!(
        configuration.read_relays,
        [
            "wss://relay.damus.io",
            "wss://relay.snort.social",
            "wss://relay.primal.net",
            "wss://nos.lol",
        ]
    );
    assert!(!configuration.read_relays.is_empty());
    assert!(!configuration.search_relays.is_empty());
    assert!(configuration
        .read_relays
        .iter()
        .chain(&configuration.search_relays)
        .all(|relay| relay.starts_with("wss://")));
}
