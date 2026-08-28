part of 'progressive_device_origin.dart';

final class ProgressiveOriginFirstChunkRendezvous {
  ProgressiveOriginFirstChunkRendezvous._(
    Set<String> paths,
    Duration timeout,
    this._activated,
    this._blocksFirstChunks,
  ) : _expectedPaths = Set.unmodifiable(_validate(paths)),
      _timeout = _validateTimeout(timeout);

  final Set<String> _expectedPaths;
  final Duration _timeout;
  final _arrivedPaths = <String>{};
  final _firstArrival = Completer<void>();
  final _reached = Completer<void>();
  final _settled = Completer<void>();
  final _released = Completer<void>();
  Timer? _watchdog;
  bool _activated;
  final bool _blocksFirstChunks;
  var _timedOut = false;

  Future<void> get firstArrival => _firstArrival.future;

  Future<void> get reached => _reached.future;

  Future<void> get settled => _settled.future;

  Set<String> get arrivedPaths => Set.unmodifiable(_arrivedPaths);

  bool get isReached => _reached.isCompleted;

  bool get isSettled => _settled.isCompleted;

  bool get isReleased => _released.isCompleted;

  bool get timedOut => _timedOut;

  void activate() {
    if (_activated || isSettled || isReleased) return;
    _activated = true;
    if (_arrivedPaths.length == _expectedPaths.length) {
      _completeReached();
    } else {
      _armWatchdog();
    }
  }

  Future<void> _afterFirstChunk(ProgressiveOriginRequest request) async {
    if (!_admitArrival(request)) return;
    _completeFirstArrival();
    if (!_activated) return;
    _observeActivatedArrival();
    await _waitForRelease();
  }

  bool _admitArrival(ProgressiveOriginRequest request) {
    if (isSettled || isReleased) return false;
    return _expectedPaths.contains(request.path) &&
        _arrivedPaths.add(request.path);
  }

  void _completeFirstArrival() {
    if (!_firstArrival.isCompleted) _firstArrival.complete();
  }

  void _observeActivatedArrival() {
    _armWatchdog();
    if (_arrivedPaths.length == _expectedPaths.length) _completeReached();
  }

  Future<void> _waitForRelease() async {
    if (_blocksFirstChunks) await _released.future;
  }

  void _armWatchdog() {
    _watchdog ??= Timer(_timeout, _failOpen);
  }

  void release() {
    _watchdog?.cancel();
    if (!_released.isCompleted) _released.complete();
  }

  void _completeReached() {
    _watchdog?.cancel();
    _reached.complete();
    _settled.complete();
    release();
  }

  void _failOpen() {
    _timedOut = true;
    _settled.complete();
    release();
  }
}
