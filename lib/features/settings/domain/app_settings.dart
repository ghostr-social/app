import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/domain/video_inventory_budget.dart';

export 'video_inventory_budget.dart';

class AppSettings {
  factory AppSettings({
    required List<RelayUrl> relays,
    required VideoInventoryBudget inventoryBudget,
    required List<BlossomServerUrl> blossomServers,
    required bool hideWatchedVideos,
  }) {
    return AppSettings._(
      List<RelayUrl>.unmodifiable(relays),
      inventoryBudget,
      List<BlossomServerUrl>.unmodifiable(blossomServers),
      hideWatchedVideos,
    );
  }

  const AppSettings._(
    this.relays,
    this.inventoryBudget,
    this.blossomServers,
    this.hideWatchedVideos,
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

  final List<RelayUrl> relays;
  final VideoInventoryBudget inventoryBudget;
  final List<BlossomServerUrl> blossomServers;
  final bool hideWatchedVideos;

  AppSettings copyWith({
    List<RelayUrl>? relays,
    VideoInventoryBudget? inventoryBudget,
    List<BlossomServerUrl>? blossomServers,
    bool? hideWatchedVideos,
  }) {
    return AppSettings(
      relays: relays ?? this.relays,
      inventoryBudget: inventoryBudget ?? this.inventoryBudget,
      blossomServers: blossomServers ?? this.blossomServers,
      hideWatchedVideos: hideWatchedVideos ?? this.hideWatchedVideos,
    );
  }
}
