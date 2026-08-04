//! The NIP-50 search relay set mirrors the Dart default exactly, order
//! included (lib/features/settings/domain/app_settings.dart
//! `AppSettings.defaultSearchRelays`, wired into the video query via
//! lib/app/production_nostr_services.dart `settings.searchRelays`).

use crate::discovery::search_queries::SEARCH_RELAY_URLS;

#[test]
fn search_relay_urls_match_the_dart_defaults_in_order() {
    assert_eq!(
        SEARCH_RELAY_URLS,
        [
            "wss://relay.nostr.band",
            "wss://nostr.wine",
            "wss://relay.noswhere.com",
            "wss://search.nos.today",
            "wss://antiprimal.net",
            "wss://relay.ditto.pub",
        ]
    );
}
