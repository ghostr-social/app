import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/delivery_config_syncing_settings_repository.dart';

import '../support/partially_failing_settings_repository.dart';
import '../support/recording_engine_updaters.dart';

void main() {
  test(
    'a partial persistence failure restores the previous settings',
    () async {
      final failure = StateError('settings write failed');
      final previous = AppSettings.defaults();
      final inner = PartiallyFailingSettingsRepository(previous, failure);
      final updater = RecordingDeliveryConfigUpdater();
      final repository = DeliveryConfigSyncingSettingsRepository(
        inner: inner,
        updateConfig: updater.call,
      );
      final next = previous.withDataUsage(DataUsageLevel.conservative);

      await expectLater(repository.save(next), throwsA(same(failure)));

      expect(inner.settings, same(previous));
      expect(updater.pushes, isEmpty);
    },
  );
}
