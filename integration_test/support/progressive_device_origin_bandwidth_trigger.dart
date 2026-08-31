part of 'progressive_device_origin.dart';

final class ProgressiveOriginBandwidthTrigger {
  ProgressiveOriginBandwidthTrigger._(
    Set<String> paths,
    this._minimumRemainingBytes,
    this._apply,
    Duration timeout,
  ) : _paths = Set.unmodifiable(_validateBandwidthPaths(paths)) {
    if (timeout <= Duration.zero) throw ArgumentError.value(timeout);
    if (_minimumRemainingBytes <= 0) {
      throw ArgumentError.value(_minimumRemainingBytes);
    }
    _watchdog = Timer(timeout, _fail);
  }

  final Set<String> _paths;
  final int _minimumRemainingBytes;
  final ProgressiveOriginLinkProfile Function() _apply;
  final _reached = Completer<void>();
  late final Timer _watchdog;
  var _timedOut = false;
  int? requestSequence;
  String? path;
  ({int start, int end})? requestRange;
  ProgressiveOriginChunkEvent? confirmedEvent;
  ProgressiveOriginLinkProfile? profile;

  Future<void> get reached => _reached.future;
  bool get isReached => _reached.isCompleted;
  bool get timedOut => _timedOut;

  void _afterChunk(
    ProgressiveOriginRequest request,
    bool hasMore,
    ProgressiveOriginChunkEvent? event,
  ) {
    if (isReached || !hasMore || event == null) return;
    if (!_paths.contains(request.path)) return;
    if (!_hasEnoughRemaining(request, event)) return;
    requestSequence = event.requestSequence;
    path = request.path;
    requestRange = request.range;
    confirmedEvent = event;
    profile = _apply();
    _watchdog.cancel();
    _reached.complete();
  }

  bool _hasEnoughRemaining(
    ProgressiveOriginRequest request,
    ProgressiveOriginChunkEvent event,
  ) {
    final responseEnd =
        request.range?.end ?? ProgressiveMp4Fixture.bytes.length;
    return responseEnd - event.end >= _minimumRemainingBytes;
  }

  void cancel() {
    _watchdog.cancel();
    if (!isReached) _completeTimeout();
  }

  void _fail() {
    _completeTimeout();
  }

  void _completeTimeout() {
    _timedOut = true;
    if (!_reached.isCompleted) _reached.complete();
  }
}

Set<String> _validateBandwidthPaths(Set<String> paths) {
  if (paths.isEmpty || paths.any((path) => !path.startsWith('/'))) {
    throw ArgumentError.value(paths);
  }
  return paths;
}
