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
    final overlapping = rangedPaths.keys.where(
      (activePath) => activePath != path,
    );
    if (overlapping.isNotEmpty) {
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

  ProgressiveRangedRequestPair? rangedByteOverlap(Iterable<String> paths) {
    final expected = paths.toSet();
    if (expected.length < 2) throw ArgumentError.value(paths);
    final candidates = requests
        .where(
          (request) =>
              expected.contains(request.path) && _hasByteInterval(request),
        )
        .toList(growable: false);
    for (var left = 0; left < candidates.length; left += 1) {
      for (var right = left + 1; right < candidates.length; right += 1) {
        final pair = (first: candidates[left], second: candidates[right]);
        if (_byteIntervalsOverlap(pair)) return pair;
      }
    }
    return null;
  }

  bool get headsRemainBlocked => _heldHeads.isNotEmpty;
}

typedef ProgressiveRangedRequestPair = ({
  ProgressiveOriginRequest first,
  ProgressiveOriginRequest second,
});

bool _hasByteInterval(ProgressiveOriginRequest request) {
  if (!_isCompletedRange(request)) return false;
  final first = request.firstByteAt;
  final last = request.lastByteAt;
  return first != null && last != null && first < last;
}

bool _isCompletedRange(ProgressiveOriginRequest request) {
  return request.outcome == ProgressiveOriginRequestOutcome.completed &&
      request.range != null &&
      request.servedBytes > 0;
}

bool _byteIntervalsOverlap(ProgressiveRangedRequestPair pair) {
  if (pair.first.path == pair.second.path) return false;
  return pair.first.firstByteAt! < pair.second.lastByteAt! &&
      pair.second.firstByteAt! < pair.first.lastByteAt!;
}
