import 'dart:io';

import 'progressive_device_origin.dart';

final class ProgressiveDeviceResources {
  ProgressiveDeviceResources._(this.origin, this._cache);

  static Future<ProgressiveDeviceResources> start() async {
    final origin = await ProgressiveDeviceOrigin.start();
    try {
      final cache = await Directory.systemTemp.createTemp(
        'ghostr-progressive-',
      );
      return ProgressiveDeviceResources._(origin, cache);
    } on Object {
      await origin.close();
      rethrow;
    }
  }

  final ProgressiveDeviceOrigin origin;
  final Directory _cache;
  var _closed = false;

  String get cachePath => _cache.path;

  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    try {
      await origin.close();
    } finally {
      if (await _cache.exists()) await _cache.delete(recursive: true);
    }
  }
}
