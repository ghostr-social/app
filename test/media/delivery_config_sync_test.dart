import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/media/delivery_config_syncing_settings_repository.dart';

import '../support/fake_app_settings_repository.dart';
import '../support/recording_engine_updaters.dart';

void main() {
  test('saving settings pushes data usage and budget to the engine', () async {
    final updater = RecordingDeliveryConfigUpdater();
    final inner = FakeAppSettingsRepository(AppSettings.defaults());
    final repository = DeliveryConfigSyncingSettingsRepository(
      inner: inner,
      updateConfig: updater.call,
    );
    final settings = AppSettings.defaults()
        .withRelays([RelayUrl.parse('wss://read.example')])
        .withSearchRelays([RelayUrl.parse('wss://search.example')])
        .withDataUsage(DataUsageLevel.aggressive)
        .withInventoryBudget(VideoInventoryBudget.fourGigabytes);

    await repository.save(settings);

    expect(inner.savedSettings, same(settings));
    expect((await repository.load()).dataUsage, DataUsageLevel.aggressive);
    final update = updater.pushes.single;
    expect(update.relayUrls, [RelayUrl.parse('wss://read.example')]);
    expect(update.searchRelayUrls, [RelayUrl.parse('wss://search.example')]);
    expect(update.dataUsage, DataUsageLevel.aggressive);
    expect(update.inventoryBudget, VideoInventoryBudget.fourGigabytes);
  });
}
