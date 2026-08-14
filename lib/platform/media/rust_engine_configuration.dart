import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';

/// Runtime settings shared by Rust discovery, publishing, and delivery.
class RustEngineConfiguration {
  RustEngineConfiguration._(
    List<RelayUrl> relayUrls,
    List<RelayUrl> searchRelayUrls,
    this.dataUsage,
    this.inventoryBudget,
  ) : relayUrls = List.unmodifiable(relayUrls),
      searchRelayUrls = List.unmodifiable(searchRelayUrls);

  factory RustEngineConfiguration.fromSettings(AppSettings settings) {
    return RustEngineConfiguration._(
      settings.relays,
      settings.searchRelays,
      settings.dataUsage,
      settings.inventoryBudget,
    );
  }

  final List<RelayUrl> relayUrls;
  final List<RelayUrl> searchRelayUrls;
  final DataUsageLevel dataUsage;
  final VideoInventoryBudget inventoryBudget;
}

/// Startup adds the engine-owned cache location to its live settings.
final class RustEngineStartConfiguration {
  const RustEngineStartConfiguration(
    this.cacheDirectory,
    this.engine, {
    this.deviceIntegrationOrigin,
  });

  final String cacheDirectory;
  final RustEngineConfiguration engine;
  final String? deviceIntegrationOrigin;

  List<RelayUrl> get relayUrls => engine.relayUrls;
  List<RelayUrl> get searchRelayUrls => engine.searchRelayUrls;
  DataUsageLevel get dataUsage => engine.dataUsage;
  VideoInventoryBudget get inventoryBudget => engine.inventoryBudget;
}
