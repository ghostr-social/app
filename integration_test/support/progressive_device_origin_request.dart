part of 'progressive_device_origin.dart';

enum ProgressiveOriginRequestOutcome {
  serving,
  headBlocked,
  completed,
  clientCanceled,
}

final class ProgressiveOriginRequest {
  ProgressiveOriginRequest(
    this.method,
    this.path,
    this.range, {
    required this.startedAt,
  });

  final String method;
  final String path;
  final ({int start, int end})? range;
  final Duration startedAt;
  var servedBytes = 0;
  var outcome = ProgressiveOriginRequestOutcome.serving;
  Duration? firstByteAt;
  Duration? lastByteAt;
  Duration? finishedAt;
  Duration? peerClosedAt;
  final _peerClosed = Completer<void>();

  bool get isPeerClosed => _peerClosed.isCompleted;

  void _recordPeerClosed(Duration elapsed) {
    peerClosedAt ??= elapsed;
    if (!_peerClosed.isCompleted) _peerClosed.complete();
  }

  void _blockHead() {
    outcome = ProgressiveOriginRequestOutcome.headBlocked;
  }

  void _recordBytes(int count, Duration elapsed) {
    firstByteAt ??= elapsed;
    lastByteAt = elapsed;
    servedBytes += count;
  }

  void _finish(ProgressiveOriginRequestOutcome value, Duration elapsed) {
    outcome = value;
    finishedAt = elapsed;
  }
}
