import 'package:ghostr/features/settings/domain/app_update_preferences.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/domain/video_inventory_budget.dart';

export 'app_update_preferences.dart';
export 'data_usage_level.dart';
export 'video_inventory_budget.dart';

class AppSettings {
  factory AppSettings({
    required List<RelayUrl> relays,
    required VideoInventoryBudget inventoryBudget,
    required List<BlossomServerUrl> blossomServers,
    List<RelayUrl>? searchRelays,
    DataUsageLevel dataUsage = DataUsageLevel.balanced,
    AppUpdatePreferences updatePreferences = AppUpdatePreferences.defaults,
  }) {
    return AppSettings._(
      List<RelayUrl>.unmodifiable(relays),
      inventoryBudget,
      List<BlossomServerUrl>.unmodifiable(blossomServers),
      List<RelayUrl>.unmodifiable(searchRelays ?? defaultSearchRelays),
      dataUsage,
      updatePreferences,
    );
  }

  const AppSettings._(
    this.relays,
    this.inventoryBudget,
    this.blossomServers,
    this.searchRelays,
    this.dataUsage,
    this.updatePreferences,
  );

  factory AppSettings.defaults() {
    return AppSettings(
      relays: [
        RelayUrl.parse('wss://relay.damus.io'),
        RelayUrl.parse('wss://relay.snort.social'),
        RelayUrl.parse('wss://relay.nostr.band'),
      ],
      inventoryBudget: VideoInventoryBudget.twoGigabytes,
      blossomServers: [BlossomServerUrl.parse('https://blossom.primal.net')],
    );
  }

  /// Relays known to implement NIP-50 full-text search.
  static final List<RelayUrl> defaultSearchRelays =
      List<RelayUrl>.unmodifiable(<RelayUrl>[
        RelayUrl.parse('wss://relay.nostr.band'),
        RelayUrl.parse('wss://nostr.wine'),
        RelayUrl.parse('wss://relay.noswhere.com'),
        RelayUrl.parse('wss://search.nos.today'),
        RelayUrl.parse('wss://antiprimal.net'),
        RelayUrl.parse('wss://relay.ditto.pub'),
      ]);

  final List<RelayUrl> relays;
  final VideoInventoryBudget inventoryBudget;
  final List<BlossomServerUrl> blossomServers;
  final List<RelayUrl> searchRelays;
  final DataUsageLevel dataUsage;
  final AppUpdatePreferences updatePreferences;

  AppSettings withRelays(List<RelayUrl> value) {
    return AppSettings(
      relays: value,
      inventoryBudget: inventoryBudget,
      blossomServers: blossomServers,
      searchRelays: searchRelays,
      dataUsage: dataUsage,
      updatePreferences: updatePreferences,
    );
  }

  AppSettings withInventoryBudget(VideoInventoryBudget value) => AppSettings(
    relays: relays,
    inventoryBudget: value,
    blossomServers: blossomServers,
    searchRelays: searchRelays,
    dataUsage: dataUsage,
    updatePreferences: updatePreferences,
  );

  AppSettings withBlossomServers(List<BlossomServerUrl> value) => AppSettings(
    relays: relays,
    inventoryBudget: inventoryBudget,
    blossomServers: value,
    searchRelays: searchRelays,
    dataUsage: dataUsage,
    updatePreferences: updatePreferences,
  );

  AppSettings withSearchRelays(List<RelayUrl> value) => AppSettings(
    relays: relays,
    inventoryBudget: inventoryBudget,
    blossomServers: blossomServers,
    searchRelays: value,
    dataUsage: dataUsage,
    updatePreferences: updatePreferences,
  );

  AppSettings withDataUsage(DataUsageLevel value) => AppSettings(
    relays: relays,
    inventoryBudget: inventoryBudget,
    blossomServers: blossomServers,
    searchRelays: searchRelays,
    dataUsage: value,
    updatePreferences: updatePreferences,
  );

  AppSettings withUpdatePreferences(AppUpdatePreferences value) => AppSettings(
    relays: relays,
    inventoryBudget: inventoryBudget,
    blossomServers: blossomServers,
    searchRelays: searchRelays,
    dataUsage: dataUsage,
    updatePreferences: value,
  );
}
