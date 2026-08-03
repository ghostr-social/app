import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('search relays persist, drop junk, and default when unsaved', () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.settings.searchRelays': [
        'wss://search.example',
        'not-a-relay',
        'wss://search.example/',
      ],
    });
    final preferences = await SharedPreferences.getInstance();
    final repository = LocalAppSettingsRepository(preferences);

    final loaded = await repository.load();
    expect(
      loaded.searchRelays.map((relay) => relay.value),
      ['wss://search.example'],
    );

    await repository.save(
      loaded.copyWith(searchRelays: [RelayUrl.parse('wss://other.example')]),
    );
    final restored = await repository.load();
    expect(
      restored.searchRelays.map((relay) => relay.value),
      ['wss://other.example'],
    );

    await preferences.remove('ghostr.settings.searchRelays');
    final fresh = await repository.load();
    expect(fresh.searchRelays, AppSettings.defaults().searchRelays);
  });
}
