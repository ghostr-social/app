import 'dart:developer';
import 'dart:io';

import 'package:ghostr/app/remote_video_delivery_source.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/data/remembering_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_inventory/data/inventory_remote_video_source.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/native_video_cache_directory.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:http/http.dart' as http;
import 'package:ndk/ndk.dart';
import 'package:path_provider/path_provider.dart';

class ProductionVideoDelivery {
  const ProductionVideoDelivery(this.inventory, this.remoteSource);

  final VideoInventoryPort inventory;
  final RemoteVideoSource remoteSource;
}

class ProductionVideoDeliveryEnvironment {
  const ProductionVideoDeliveryEnvironment({
    required this.canonicalSource,
    required this.supportDirectoryProvider,
    required this.downloader,
    required this.gateway,
  });

  factory ProductionVideoDeliveryEnvironment.production(Ndk ndk) {
    return ProductionVideoDeliveryEnvironment(
      canonicalSource: NdkVideoRemoteSource(NdkNostrVideoEventQuery(ndk)),
      supportDirectoryProvider: getApplicationSupportDirectory,
      downloader: HttpVideoFileDownloader(http.Client()),
      gateway: FfiVideoGateway(),
    );
  }

  final RemoteVideoSource canonicalSource;
  final CacheDirectoryProvider supportDirectoryProvider;
  final VideoFileDownloader downloader;
  final FfiVideoGateway gateway;
}

Future<ProductionVideoDelivery> buildProductionVideoDelivery(
  AppSettings settings,
  ProductionVideoDeliveryEnvironment environment,
) async {
  final plan = VideoDeliveryPlan.fromSettings(settings);
  final root = await environment.supportDirectoryProvider();
  final directories = _VideoDirectories(root);
  final inventory = await _buildVideoInventory(
    plan,
    directories.dartCache,
    environment.downloader,
  );
  await NativeVideoCacheDirectory(directories.nativeCache).initialize();
  final snapshot = NostrVideoSnapshot();
  final canonical = RememberingRemoteVideoSource(
    environment.canonicalSource,
    snapshot,
  );
  final native = await _buildRemoteVideoSource(
    plan,
    directories.nativeCache.path,
    snapshot,
    environment.gateway,
  );
  final source = buildRemoteVideoDeliverySource(
    primary: InventoryRemoteVideoSource(
      source: canonical,
      inventory: inventory,
    ),
    nativeFallback: native,
  );
  return ProductionVideoDelivery(inventory, source);
}

Future<VideoInventoryPort> _buildVideoInventory(
  VideoDeliveryPlan plan,
  Directory directory,
  VideoFileDownloader downloader,
) async {
  final store = FileVideoCacheStore(
    directoryProvider: () async => directory,
    downloader: downloader,
    maxBytes: plan.dartCacheBytes,
  );
  await store.initialize();
  return SmartVideoInventory(
    store: store,
    maxParallelDownloads: 3,
    maxPreparedVideos: 8,
  );
}

Future<RemoteVideoSource> _buildRemoteVideoSource(
  VideoDeliveryPlan plan,
  String cacheDirectory,
  NostrVideoSnapshot snapshot,
  FfiVideoGateway gateway,
) async {
  final result = await gateway.start(plan, cacheDirectory);
  return switch (result) {
    VideoGatewayStarted(:final endpoint) => FfiVideoRemoteSource(
        gatewayBaseUrl: 'http://$endpoint',
        snapshotLoader: snapshot.read,
      ),
    VideoGatewayFailed(:final message) => _reportedFailure(message),
  };
}

DisabledRemoteVideoSource _reportedFailure(String message) {
  log(message, name: 'ghostr.gateway');
  return _disabledGateway();
}

DisabledRemoteVideoSource _disabledGateway() {
  return const DisabledRemoteVideoSource(
    'The embedded Nostr gateway is unavailable.',
  );
}

class _VideoDirectories {
  _VideoDirectories(Directory root)
      : dartCache = Directory(_child(root, 'video_inventory')),
        nativeCache = Directory(_child(root, 'native_video_inventory'));

  final Directory dartCache;
  final Directory nativeCache;

  static String _child(Directory root, String name) {
    return '${root.path}${Platform.pathSeparator}$name';
  }
}
