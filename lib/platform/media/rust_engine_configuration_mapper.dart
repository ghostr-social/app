import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/media/rust_engine_configuration.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';

FfiEngineConfiguration ffiEngineConfiguration(
  RustEngineConfiguration configuration,
) {
  return FfiEngineConfiguration(
    readRelayUrls: _relayValues(configuration.relayUrls),
    searchRelayUrls: _relayValues(configuration.searchRelayUrls),
    dataUsage: _dataUsage(configuration.dataUsage),
    maxStorageBytes: BigInt.from(configuration.inventoryBudget.bytes),
  );
}

FfiDataUsageLevel _dataUsage(DataUsageLevel level) {
  return switch (level) {
    DataUsageLevel.conservative => FfiDataUsageLevel.conservative,
    DataUsageLevel.balanced => FfiDataUsageLevel.balanced,
    DataUsageLevel.aggressive => FfiDataUsageLevel.aggressive,
  };
}

List<String> _relayValues(List<RelayUrl> relays) {
  return relays.map((relay) => relay.value).toList(growable: false);
}
