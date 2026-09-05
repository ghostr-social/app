part of 'warp_cache_pressure_scenario.dart';

const _cachePressureBudgetBytes = 700 * 1024;
const _maximumPressureHandoffs = 3;

Future<void> _configureCachePressure(WarpLongSessionScenario session) {
  return ffiSetDeliveryConfig(
    configuration: FfiEngineConfiguration(
      readRelayUrls: [session.relay.uri.toString()],
      searchRelayUrls: const [],
      dataUsage: FfiDataUsageLevel.aggressive,
      maxStorageBytes: BigInt.from(_cachePressureBudgetBytes),
    ),
  );
}
