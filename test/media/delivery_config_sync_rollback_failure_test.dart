import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/delivery_config_syncing_settings_repository.dart';

import '../support/failing_rollback_settings_repository.dart';
import '../support/recording_engine_updaters.dart';

void main() {
  test('rollback failure surfaces persisted/live divergence', () async {
    final engineError = StateError('engine offline');
    final updater = RecordingDeliveryConfigUpdater()..failure = engineError;
    final inner = FailingRollbackSettingsRepository(AppSettings.defaults());
    final repository = DeliveryConfigSyncingSettingsRepository(
      inner: inner,
      updateConfig: updater.call,
    );

    await expectLater(
      repository.save(
        AppSettings.defaults().copyWith(
          dataUsage: DataUsageLevel.conservative,
        ),
      ),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          contains('may differ'),
        ),
      ),
    );
  });
}
