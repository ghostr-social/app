import 'dart:io';

import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:path_provider/path_provider.dart';

import 'live_feed_probe.dart';
import 'live_video_log.dart';
import 'live_social_probe.dart';

final class LiveVideoEnvironment {
  LiveVideoEnvironment(this.log);
  final LiveVideoLog log;
  ProductionVideoDelivery? delivery;
  List<String> relays = const [];

  ProductionDependenciesEnvironment build() {
    final production = ProductionDependenciesEnvironment.production();
    return ProductionDependenciesEnvironment(
      preferencesLoader: production.preferencesLoader,
      nostrServicesBuilder: (settings) =>
          liveNostrServices(production.nostrServicesBuilder(settings), log),
      videoDeliveryBuilder: _delivery,
      watchHistoryDatabaseLoader: production.watchHistoryDatabaseLoader,
      appUpdateBuilder: production.appUpdateBuilder,
    );
  }

  Future<ProductionVideoDelivery> _delivery(
    AppSettings settings,
    ProductionNostrServices nostr,
  ) async {
    relays = {
      ...settings.relays,
      ...settings.searchRelays,
    }.map((relay) => relay.value).toList();
    log.add('configuration', {
      'relays': settings.relays.map((r) => r.value).toList(),
      'searchRelays': settings.searchRelays.map((r) => r.value).toList(),
      'dataUsage': settings.dataUsage.name,
      'cacheBudget': settings.inventoryBudget.name,
      'coldCache': const bool.fromEnvironment('LIVE_COLD_CACHE'),
    });
    final normal = ProductionVideoDeliveryEnvironment.production(
      signedInViewer(nostr.eventClient),
    );
    delivery = await buildProductionVideoDelivery(
      settings,
      ProductionVideoDeliveryEnvironment(
        source: RustFeedRemoteSource(
          port: LiveFeedProbe(log),
          viewer: signedInViewer(nostr.eventClient),
        ),
        adapters: _adapters(normal.adapters),
        playbackCapabilities: normal.playbackCapabilities,
      ),
    );
    log.add('delivery_started', {});
    return delivery!;
  }

  ProductionVideoDeliveryAdapters _adapters(
    ProductionVideoDeliveryAdapters normal,
  ) {
    if (!const bool.fromEnvironment('LIVE_COLD_CACHE')) return normal;
    return ProductionVideoDeliveryAdapters(
      supportDirectoryProvider: _coldDirectory,
      gateway: normal.gateway,
      hlsPlaybackGateway: normal.hlsPlaybackGateway,
      preparationUpdates: normal.preparationUpdates,
      networkStatus: normal.networkStatus,
      applyNetworkStatus: normal.applyNetworkStatus,
    );
  }

  Future<Directory> _coldDirectory() async {
    final support = await getApplicationSupportDirectory();
    const requested = String.fromEnvironment('LIVE_COLD_CACHE_KEY');
    final key = requested.isEmpty
        ? '${DateTime.now().microsecondsSinceEpoch}'
        : requested;
    if (!RegExp(r'^[a-zA-Z0-9_-]+$').hasMatch(key)) {
      throw ArgumentError('Invalid cache key.');
    }
    log.add('isolated_cache', {'key': key});
    return Directory('${support.path}/live-video-cold-$key');
  }
}
