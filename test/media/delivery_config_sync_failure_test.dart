import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/delivery_config_syncing_settings_repository.dart';

import '../support/fake_app_settings_repository.dart';
import '../support/recording_engine_updaters.dart';

void main() {
  test('a failed engine push never fails the settings save', () async {
    final updater = RecordingDeliveryConfigUpdater()
      ..failure = StateError('engine offline');
    final inner = FakeAppSettingsRepository(AppSettings.defaults());
    final repository = DeliveryConfigSyncingSettingsRepository(
      inner: inner,
      updateConfig: updater.call,
    );
    final settings = AppSettings.defaults().copyWith(
      dataUsage: DataUsageLevel.conservative,
    );

    await repository.save(settings);

    expect(inner.savedSettings, same(settings));
    expect(updater.pushes, isEmpty);
  });
}
