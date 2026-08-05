import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/platform/media/rust_engine_configuration.dart';
import 'package:ghostr/platform/media/rust_engine_configuration_mapper.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';

typedef RustDeliveryConfigUpdater = Future<void> Function(
  RustEngineConfiguration configuration,
);
typedef RustDeliveryConfigSetter = Future<void> Function({
  required FfiEngineConfiguration configuration,
});

const settingsConfigDivergenceFailure = AppFailure(
  'Persisted settings and the live engine may differ; restart the app.',
);

/// Persists settings and pushes the matching live Rust configuration.
///
/// A failed write or engine update restores the previous persisted settings
/// so the next startup cannot silently diverge from the running engine.
final class DeliveryConfigSyncingSettingsRepository
    implements AppSettingsRepository {
  const DeliveryConfigSyncingSettingsRepository({
    required AppSettingsRepository inner,
    RustDeliveryConfigUpdater updateConfig = updateRustEngineConfiguration,
  })  : _inner = inner,
        _updateConfig = updateConfig;

  final AppSettingsRepository _inner;
  final RustDeliveryConfigUpdater _updateConfig;

  @override
  Future<AppSettings> load() => _inner.load();

  @override
  Future<void> save(AppSettings settings) async {
    final previous = await _inner.load();
    try {
      await _inner.save(settings);
      await _updateConfig(RustEngineConfiguration.fromSettings(settings));
    } on Object catch (error, stackTrace) {
      final restored = await _restore(previous);
      if (!restored) throw settingsConfigDivergenceFailure;
      Error.throwWithStackTrace(error, stackTrace);
    }
  }

  Future<bool> _restore(AppSettings previous) async {
    try {
      await _inner.save(previous);
      return true;
    } on Object catch (error, stackTrace) {
      log('Settings rollback failed after a rejected settings transaction.',
          name: 'ghostr.video.config', error: error, stackTrace: stackTrace);
      return false;
    }
  }
}

Future<void> updateRustEngineConfiguration(
  RustEngineConfiguration configuration, {
  RustDeliveryConfigSetter setConfig = ffiSetDeliveryConfig,
}) {
  return setConfig(
    configuration: ffiEngineConfiguration(configuration),
  );
}
