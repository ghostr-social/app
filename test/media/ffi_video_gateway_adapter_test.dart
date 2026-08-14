import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';

void main() {
  test('Rust engine starter maps cache and delivery configuration', () async {
    String? receivedDirectory;
    FfiEngineConfiguration? receivedConfiguration;
    final engine = RustEngineConfiguration.fromSettings(AppSettings.defaults());

    final endpoint = await startRustEngine(
      RustEngineStartConfiguration('/cache/native', engine),
      ffiStart:
          ({
            required cacheDirectory,
            required configuration,
            required deviceIntegrationOrigin,
          }) async {
            receivedDirectory = cacheDirectory;
            receivedConfiguration = configuration;
            expect(deviceIntegrationOrigin, isNull);
            return '127.0.0.1:3000';
          },
    );

    expect(endpoint, '127.0.0.1:3000');
    expect(receivedDirectory, '/cache/native');
    expect(
      receivedConfiguration?.maxStorageBytes,
      BigInt.from(engine.inventoryBudget.bytes),
    );
  });
}
