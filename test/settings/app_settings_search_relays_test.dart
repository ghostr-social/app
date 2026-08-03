import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';

void main() {
  test('settings carry NIP-50 search relays with sensible defaults', () {
    final defaults = AppSettings.defaults();

    expect(
      defaults.searchRelays.map((relay) => relay.value),
      [
        'wss://relay.nostr.band',
        'wss://nostr.wine',
        'wss://relay.noswhere.com',
        'wss://search.nos.today',
        'wss://antiprimal.net',
        'wss://relay.ditto.pub',
      ],
    );

    final custom = defaults.copyWith(
      searchRelays: [RelayUrl.parse('wss://search.example')],
    );
    expect(
      custom.searchRelays.map((relay) => relay.value),
      ['wss://search.example'],
    );
    expect(custom.relays, defaults.relays);
    expect(
      custom.copyWith(hideWatchedVideos: false).searchRelays,
      custom.searchRelays,
    );
  });
}
