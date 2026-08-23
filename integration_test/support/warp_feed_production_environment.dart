import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ndk/ndk.dart';
import 'package:sembast/sembast_memory.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'progressive_device_resources.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_production_delivery.dart';
import 'warp_feed_rust_probe.dart';

final class WarpFeedProductionCapture {
  ProductionNostrServices? nostr;
  ProductionVideoDelivery? delivery;
  final rustProbe = WarpFeedRustProbe();
}

ProductionDependenciesEnvironment warpFeedProductionEnvironment(
  Ndk ndk,
  ProgressiveDeviceResources resources,
  WarpFeedPreparationProbe preparation,
  WarpFeedProductionCapture capture,
) {
  return ProductionDependenciesEnvironment(
    preferencesLoader: SharedPreferences.getInstance,
    nostrServicesBuilder: (settings) {
      final nostr = buildProductionNostrServices(
        settings,
        ndkBuilder: () => ndk,
      );
      capture.nostr = nostr;
      return nostr;
    },
    videoDeliveryBuilder: (settings, nostr) async {
      final delivery = await buildWarpFeedProductionDelivery((
        settings: settings,
        nostr: nostr,
        resources: resources,
        preparation: preparation,
        rustProbe: capture.rustProbe,
      ));
      capture.delivery = delivery;
      return delivery;
    },
    watchHistoryDatabaseLoader: () => databaseFactoryMemory.openDatabase(
      'warp-feed-${DateTime.now().microsecondsSinceEpoch}',
      mode: DatabaseMode.create,
    ),
  );
}
