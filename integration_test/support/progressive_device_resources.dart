import 'dart:io';

import 'progressive_device_origin.dart';

final class ProgressiveDeviceResources {
  ProgressiveDeviceResources._(
    this.origin,
    this._cache, {
    required bool deleteCacheOnClose,
  }) : _deleteCacheOnClose = deleteCacheOnClose;

  static Future<ProgressiveDeviceResources> start({
    ProgressiveOriginPacing pacing =
        const ProgressiveOriginPacing.perResponseDelay(Duration.zero),
    ProgressiveOriginValidator validator = ProgressiveOriginValidator.none,
    Map<String, ProgressiveOriginRangeSemantics> rangeSemanticsById = const {},
  }) async {
    final origin = await ProgressiveDeviceOrigin.start(
      pacing: pacing,
      validator: validator,
      rangeSemanticsById: rangeSemanticsById,
    );
    try {
      final cache = await Directory.systemTemp.createTemp(
        'ghostr-progressive-',
      );
      return ProgressiveDeviceResources._(
        origin,
        cache,
        deleteCacheOnClose: true,
      );
    } on Object {
      await origin.close();
      rethrow;
    }
  }

  static Future<ProgressiveDeviceResources> startPersistent(
    Directory cacheDirectory, {
    int originPort = 0,
    ProgressiveOriginAvailability originAvailability =
        ProgressiveOriginAvailability.available,
    ProgressiveOriginValidator validator = ProgressiveOriginValidator.none,
  }) async {
    final origin = await ProgressiveDeviceOrigin.start(
      port: originPort,
      availability: originAvailability,
      validator: validator,
    );
    try {
      await cacheDirectory.create(recursive: true);
      return ProgressiveDeviceResources._(
        origin,
        cacheDirectory,
        deleteCacheOnClose: false,
      );
    } on Object {
      await origin.close();
      rethrow;
    }
  }

  final ProgressiveDeviceOrigin origin;
  final Directory _cache;
  final bool _deleteCacheOnClose;
  var _closed = false;

  String get cachePath => _cache.path;

  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    try {
      await origin.close();
    } finally {
      if (_deleteCacheOnClose && await _cache.exists()) {
        await _cache.delete(recursive: true);
      }
    }
  }
}
