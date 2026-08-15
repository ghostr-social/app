import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/media/rust_engine_configuration.dart';
import 'package:ghostr/platform/media/rust_engine_configuration_mapper.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';

void main() {
  test('maps every live engine setting into the generated FFI shape', () {
    const usages = <DataUsageLevel, FfiDataUsageLevel>{
      DataUsageLevel.conservative: FfiDataUsageLevel.conservative,
      DataUsageLevel.balanced: FfiDataUsageLevel.balanced,
      DataUsageLevel.aggressive: FfiDataUsageLevel.aggressive,
    };

    for (final usage in usages.entries) {
      final settings = AppSettings.defaults()
          .withRelays([RelayUrl.parse('wss://read.example')])
          .withSearchRelays([RelayUrl.parse('wss://search.example')])
          .withDataUsage(usage.key)
          .withInventoryBudget(VideoInventoryBudget.fourGigabytes);
      final mapped = ffiEngineConfiguration(
        RustEngineConfiguration.fromSettings(settings),
      );

      expect(mapped.readRelayUrls, ['wss://read.example']);
      expect(mapped.searchRelayUrls, ['wss://search.example']);
      expect(mapped.dataUsage, usage.value);
      expect(
        mapped.maxStorageBytes,
        BigInt.from(VideoInventoryBudget.fourGigabytes.bytes),
      );
    }
  });
}
