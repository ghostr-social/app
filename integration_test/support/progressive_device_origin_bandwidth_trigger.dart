part of 'progressive_device_origin.dart';

final class ProgressiveOriginBandwidthTrigger {
  ProgressiveOriginBandwidthTrigger._(
    Set<String> paths,
    this._apply,
    Duration timeout,
  ) : _paths = Set.unmodifiable(_validateBandwidthPaths(paths)) {
    if (timeout <= Duration.zero) throw ArgumentError.value(timeout);
    _watchdog = Timer(timeout, _fail);
  }

  final Set<String> _paths;
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
    requestSequence = event.requestSequence;
    path = request.path;
    requestRange = request.range;
    confirmedEvent = event;
    profile = _apply();
    _watchdog.cancel();
    _reached.complete();
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
