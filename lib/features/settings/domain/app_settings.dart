import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/domain/video_inventory_budget.dart';

export 'data_usage_level.dart';
export 'video_inventory_budget.dart';

class AppSettings {
  factory AppSettings({
    required List<RelayUrl> relays,
    required VideoInventoryBudget inventoryBudget,
    required List<BlossomServerUrl> blossomServers,
    required bool hideWatchedVideos,
    List<RelayUrl>? searchRelays,
    DataUsageLevel dataUsage = DataUsageLevel.balanced,
  }) {
    return AppSettings._(
      List<RelayUrl>.unmodifiable(relays),
      inventoryBudget,
      List<BlossomServerUrl>.unmodifiable(blossomServers),
      hideWatchedVideos,
      List<RelayUrl>.unmodifiable(searchRelays ?? defaultSearchRelays),
      dataUsage,
    );
  }

  const AppSettings._(
    this.relays,
    this.inventoryBudget,
    this.blossomServers,
    this.hideWatchedVideos,
    this.searchRelays,
    this.dataUsage,
  );

  factory AppSettings.defaults() {
    return AppSettings(
      relays: [
        RelayUrl.parse('wss://relay.damus.io'),
        RelayUrl.parse('wss://relay.snort.social'),
        RelayUrl.parse('wss://relay.nostr.band'),
      ],
      inventoryBudget: VideoInventoryBudget.twoGigabytes,
      blossomServers: [
        BlossomServerUrl.parse('https://blossom.primal.net'),
      ],
      hideWatchedVideos: true,
    );
  }

  /// Relays known to implement NIP-50 full-text search.
  static final List<RelayUrl> defaultSearchRelays = List<RelayUrl>.unmodifiable(
    <RelayUrl>[
      RelayUrl.parse('wss://relay.nostr.band'),
      RelayUrl.parse('wss://nostr.wine'),
      RelayUrl.parse('wss://relay.noswhere.com'),
      RelayUrl.parse('wss://search.nos.today'),
    ],
  );

  final List<RelayUrl> relays;
  final VideoInventoryBudget inventoryBudget;
  final List<BlossomServerUrl> blossomServers;
  final bool hideWatchedVideos;
  final List<RelayUrl> searchRelays;
  final DataUsageLevel dataUsage;

  AppSettings copyWith({
    List<RelayUrl>? relays,
    VideoInventoryBudget? inventoryBudget,
    List<BlossomServerUrl>? blossomServers,
    bool? hideWatchedVideos,
    List<RelayUrl>? searchRelays,
    DataUsageLevel? dataUsage,
  }) {
    return AppSettings(
      relays: relays ?? this.relays,
      inventoryBudget: inventoryBudget ?? this.inventoryBudget,
      blossomServers: blossomServers ?? this.blossomServers,
      hideWatchedVideos: hideWatchedVideos ?? this.hideWatchedVideos,
      searchRelays: searchRelays ?? this.searchRelays,
      dataUsage: dataUsage ?? this.dataUsage,
    );
  }
}
