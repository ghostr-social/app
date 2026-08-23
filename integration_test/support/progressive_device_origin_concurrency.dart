part of 'progressive_device_origin.dart';

final class _ProgressiveOriginConcurrency {
  var active = 0;
  var maximum = 0;
  final rangedPaths = <String, int>{};
  var parallelRangedVideos = false;

  void started(String path, ({int start, int end})? range) {
    active += 1;
    if (active > maximum) maximum = active;
    if (range == null) return;
    if (rangedPaths.keys.any((activePath) => activePath != path)) {
      parallelRangedVideos = true;
    }
    rangedPaths.update(path, (count) => count + 1, ifAbsent: () => 1);
  }

  void finished(String path, ({int start, int end})? range) {
    active -= 1;
    if (range == null) return;
    final remaining = rangedPaths[path]! - 1;
    if (remaining == 0) {
      rangedPaths.remove(path);
    } else {
      rangedPaths[path] = remaining;
    }
  }
}

extension ProgressiveOriginConcurrencyMetrics on ProgressiveDeviceOrigin {
  int get maximumConcurrentResponses => _concurrency.maximum;

  bool get hadParallelRangedVideos => _concurrency.parallelRangedVideos;

  bool get headsRemainBlocked => _heldHeads.isNotEmpty;
}
