import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/native_video_cache_directory.dart';

/// Prepares the on-disk media partitions and starts the Rust engine.
///
/// The Dart download stack is retired (plan §6), so the only inventory
/// left is the engine's own.
Future<VideoGatewayStartResult> initializeProductionVideoDeliveryInfrastructure({
  required AppSettings settings,
  required CacheDirectoryProvider directoryProvider,
  required FfiVideoGateway gateway,
}) async {
  final directories = _VideoDeliveryDirectories(await directoryProvider());
  await _removeRetiredDartCache(directories.dartCache);
  return _startNativeDelivery(settings, directories.nativeCache, gateway);
}

Future<VideoGatewayStartResult> _startNativeDelivery(
  AppSettings settings,
  Directory directory,
  FfiVideoGateway gateway,
) async {
  try {
    await NativeVideoCacheDirectory(directory).initialize();
    return gateway.start(settings, directory.path);
  } on AppFailure catch (failure) {
    return VideoGatewayFailed(failure.message);
  }
}

/// Nothing drains the pre-migration Dart partition now that its store is
/// deleted, so startup reclaims it once and for all. A device that
/// refuses the delete must still reach playback: the engine caches
/// elsewhere.
Future<void> _removeRetiredDartCache(Directory directory) async {
  try {
    if (await directory.exists()) {
      await directory.delete(recursive: true);
    }
  } on FileSystemException {
    return;
  }
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
