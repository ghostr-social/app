import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';
import 'package:ghostr/platform/media/native_video_cache_directory.dart';

final class ProductionVideoDeliveryInfrastructure {
  const ProductionVideoDeliveryInfrastructure({
    required this.inventory,
    required this.gatewayResult,
  });

  final VideoInventoryPort inventory;
  final VideoGatewayStartResult gatewayResult;
}

Future<ProductionVideoDeliveryInfrastructure>
    initializeProductionVideoDeliveryInfrastructure({
  required AppSettings settings,
  required CacheDirectoryProvider directoryProvider,
  required VideoFileDownloader downloader,
  required FfiVideoGateway gateway,
}) async {
  final plan = VideoDeliveryPlan.fromSettings(settings);
  final directories = _VideoDeliveryDirectories(await directoryProvider());
  final inventory =
      await _buildInventory(plan, directories.dartCache, downloader);
  final result = await _startNativeDelivery(
    plan,
    directories.nativeCache,
    gateway,
  );
  return ProductionVideoDeliveryInfrastructure(
    inventory: inventory,
    gatewayResult: result,
  );
}

Future<VideoGatewayStartResult> _startNativeDelivery(
  VideoDeliveryPlan plan,
  Directory directory,
  FfiVideoGateway gateway,
) async {
  try {
    await NativeVideoCacheDirectory(directory).initialize();
    return gateway.start(plan, directory.path);
  } on AppFailure catch (failure) {
    return VideoGatewayFailed(failure.message);
  }
}

Future<VideoInventoryPort> _buildInventory(
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

final class _VideoDeliveryDirectories {
  _VideoDeliveryDirectories(Directory root)
      : dartCache = Directory(_child(root, 'video_inventory')),
        nativeCache = Directory(_child(root, 'native_video_inventory'));

  final Directory dartCache;
  final Directory nativeCache;

  static String _child(Directory root, String name) {
    return '${root.path}${Platform.pathSeparator}$name';
  }
}
