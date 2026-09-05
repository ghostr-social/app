part of 'warp_unsupported_hls_rescue_scenario.dart';

extension _WarpUnsupportedHlsRescueWait on _WarpUnsupportedHlsRescueDriver {
  Future<_UnsupportedHlsEvidence> _waitForDecodedAlternate() async {
    await _wait(_hasDecodedAlternate, timeout: const Duration(seconds: 40));
    final rescue = _alternateFocus()!;
    final before = graph.telemetry.probe.latestPositionFor(rescue)!;
    await _pumpFor(const Duration(seconds: 1));
    await _wait(() => _advancedPast(rescue, before));
    return (
      failedFocus: _failedFocus()!,
      failure: _failure()!,
      alternateFocus: rescue,
      before: before,
      after: graph.telemetry.probe.latestPositionFor(rescue)!,
    );
  }

  bool _hasDecodedAlternate() {
    final rescue = _alternateFocus();
    if (_failure() == null || _failedFocus() == null || rescue == null) {
      return false;
    }
    final probe = graph.telemetry.probe;
    return probe.firstFrameLatency(rescue) != null &&
        probe.playingLatency(rescue) != null &&
        probe.latestPositionFor(rescue) != null;
  }

  VideoDeliverySnapshot? _failure() {
    VideoDeliverySnapshot? found;
    for (final item in graph.deliveryProbe.observations) {
      final snapshot = item.snapshot;
      if (snapshot.deliveryId == failedDeliveryId &&
          snapshot.phase == VideoDeliveryPhase.failed) {
        found = snapshot;
      }
    }
    return found;
  }

  PlaybackFocus? _failedFocus() =>
      graph.focus.publishedFor(runtime.events[0].id);

  PlaybackFocus? _alternateFocus() => graph.focus.occurrenceAfter(
    runtime.events[1].id,
    0,
    cause: FeedFocusCause.userNavigation,
  );

  bool _advancedPast(PlaybackFocus focus, Duration before) =>
      (graph.telemetry.probe.latestPositionFor(focus) ?? Duration.zero) >
      before;

  Future<void> _wait(
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await _tick();
    }
    if (!condition()) fail(_timeoutEvidence(timeout));
  }

  Future<void> _pumpFor(Duration duration) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < duration) {
      await _tick();
    }
  }

  Future<void> _tick() async {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    _sampleCapacity();
  }

  void _sampleCapacity() {
    final mounted = find.byType(VideoPlayer, skipOffstage: false).evaluate();
    peakMountedPlayers = mounted.length > peakMountedPlayers
        ? mounted.length
        : peakMountedPlayers;
    final capacity = videoPlaybackCapacityOf(graph.playback).inUse;
    peakControllerCapacity = capacity > peakControllerCapacity
        ? capacity
        : peakControllerCapacity;
    unavailableWasVisible |= find
        .text('Video unavailable')
        .evaluate()
        .isNotEmpty;
  }

  String _timeoutEvidence(Duration timeout) =>
      'Unsupported-HLS navigation timed out after $timeout; '
      'state=${graph.cubit.state.runtimeType}, '
      'delivery=${graph.deliveryProbe.evidence}, '
      'hlsRequests=${progressive.encryptedHlsRequests.length}, '
      'focus=${graph.focus.occurrences.map((item) => item.cause.name)}.';
}
