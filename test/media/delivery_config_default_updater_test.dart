import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/delivery_config_syncing_settings_repository.dart';
import 'package:ghostr/platform/media/rust_engine_configuration.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';

void main() {
  test('default delivery updater maps configuration onto the Rust bridge',
      () async {
    FfiEngineConfiguration? received;
    final configuration = RustEngineConfiguration.fromSettings(
      AppSettings.defaults(),
    );

    await updateRustEngineConfiguration(
      configuration,
      setConfig: ({required configuration}) async => received = configuration,
    );

    expect(
      received?.maxStorageBytes,
      BigInt.from(configuration.inventoryBudget.bytes),
    );
  });
}
