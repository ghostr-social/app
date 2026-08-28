part of 'progressive_device_origin.dart';

final class ProgressiveOriginChunkGate {
  ProgressiveOriginChunkGate._(
    Set<String> paths,
    this._afterChunks,
    Duration timeout,
  ) : _paths = Set.unmodifiable(paths) {
    if (paths.isEmpty || paths.any((path) => !path.startsWith('/'))) {
      throw ArgumentError.value(paths);
    }
    if (_afterChunks <= 0) throw ArgumentError.value(_afterChunks);
    if (timeout <= Duration.zero) throw ArgumentError.value(timeout);
    _watchdog = Timer(timeout, _failOpen);
  }

  final Set<String> _paths;
  final int _afterChunks;
  final _counts = <int, int>{};
  final _reached = Completer<void>();
  final _released = Completer<void>();
  late final Timer _watchdog;
  var _timedOut = false;
  int? requestSequence;
  String? path;
  ({int start, int end})? requestRange;

  Future<void> get reached => _reached.future;
  bool get isReached => _reached.isCompleted;
  bool get isReleased => _released.isCompleted;
  bool get timedOut => _timedOut;

  Future<void> _afterChunk(
    ProgressiveOriginRequest request,
    int sequence,
    bool hasMore,
  ) async {
    if (isReleased ||
        requestSequence != null ||
        !_paths.contains(request.path)) {
      return;
    }
    final count = _counts.update(
      sequence,
      (value) => value + 1,
      ifAbsent: () => 1,
    );
    if (count < _afterChunks || !hasMore) return;
    requestSequence = sequence;
    path = request.path;
    requestRange = request.range;
    _watchdog.cancel();
    _reached.complete();
    await _released.future;
  }

  void release() {
    _watchdog.cancel();
    if (!_released.isCompleted) _released.complete();
  }

  void _failOpen() {
    _timedOut = true;
    if (!_reached.isCompleted) _reached.complete();
    release();
  }
}
