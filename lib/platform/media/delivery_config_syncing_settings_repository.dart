import 'dart:developer';

import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';

typedef RustDeliveryConfigUpdater = Future<void> Function({
  required String dataUsage,
  required BigInt maxStorageBytes,
});

/// Saves settings and then pushes the data-usage level and the full
/// storage budget to the running Rust engine via
/// `ffi_set_delivery_config`, so knob changes apply without a restart.
/// A failed push never fails the save: the engine re-reads the same
/// settings at its next start.
final class DeliveryConfigSyncingSettingsRepository
    implements AppSettingsRepository {
  const DeliveryConfigSyncingSettingsRepository({
    required AppSettingsRepository inner,
    RustDeliveryConfigUpdater updateConfig = ffiSetDeliveryConfig,
  })  : _inner = inner,
        _updateConfig = updateConfig;

  final AppSettingsRepository _inner;
  final RustDeliveryConfigUpdater _updateConfig;

  @override
  Future<AppSettings> load() => _inner.load();

  @override
  Future<void> save(AppSettings settings) async {
    await _inner.save(settings);
    await _pushDeliveryConfig(settings);
  }

  Future<void> _pushDeliveryConfig(AppSettings settings) async {
    try {
      await _updateConfig(
        dataUsage: settings.dataUsage.name,
        maxStorageBytes: BigInt.from(settings.inventoryBudget.bytes),
      );
    } on Object catch (error, stackTrace) {
      log('Delivery config did not reach the engine.',
          name: 'ghostr.video.config', error: error, stackTrace: stackTrace);
    }
  }
}
