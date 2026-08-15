import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test(
    'loads canonical relay inventory settings and persists changes',
    () async {
      SharedPreferences.setMockInitialValues({
        'ghostr.settings.relays': [
          'wss://relay.example',
          'not-a-relay',
          'wss://relay.example/',
        ],
        'ghostr.settings.inventoryBudget': 'fourGigabytes',
        'ghostr.settings.blossomServers': [
          'https://blossom.primal.net/',
          'not-a-server',
        ],
      });
      final preferences = await SharedPreferences.getInstance();
      final repository = LocalAppSettingsRepository(preferences);

      final loaded = await repository.load();

      expect(loaded.relays.map((relay) => relay.value), [
        'wss://relay.example',
      ]);
      expect(loaded.inventoryBudget, VideoInventoryBudget.fourGigabytes);
      expect(loaded.blossomServers.map((server) => server.value), [
        'https://blossom.primal.net',
      ]);

      await repository.save(
        loaded.withInventoryBudget(
          VideoInventoryBudget.twoHundredFiftySixMegabytes,
        ),
      );
      final restored = await repository.load();
      expect(
        restored.inventoryBudget,
        VideoInventoryBudget.twoHundredFiftySixMegabytes,
      );
    },
  );
}
