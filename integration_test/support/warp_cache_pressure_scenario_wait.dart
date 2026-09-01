part of 'warp_cache_pressure_scenario.dart';

extension _WarpCachePressureWait on _WarpCachePressureDriver {
  Future<void> _waitForDecodedPlayback(PlaybackFocus focus) async {
    await _wait(() => _hasDecodedPlayback(focus));
    _expectExactPresentation(focus);
  }

  bool _hasDecodedPlayback(PlaybackFocus focus) {
    final probe = graph.telemetry.probe;
    final presentation = probe.presentationFor(focus);
    final session = probe.sessionFor(focus);
    if (presentation == null || session == null) return false;
    final stage = graph.playerStages.forPresentation(
      session.deliveryId,
      presentation.elapsed,
    );
    return stage?.firstFrameAt != null && probe.playingLatency(focus) != null;
  }

  void _expectExactPresentation(PlaybackFocus focus) {
    final probe = graph.telemetry.probe;
    final session = probe.sessionFor(focus)!;
    final expected = graph.focus.deliveryForEvent(focus.videoId.value);
    expect(session.videoId, focus.videoId);
    expect(session.deliveryId, expected);
    final presented = probe.presentationFor(focus)!;
    final stage = graph.playerStages.forPresentation(
      session.deliveryId,
      presented.elapsed,
    );
    expect(stage?.authority.deliveryId, session.deliveryId);
  }

  Future<void> _wait(bool Function() condition, {Duration? timeout}) async {
    final watch = Stopwatch()..start();
    final limit = timeout ?? const Duration(seconds: 30);
    while (!condition() && watch.elapsed < limit) {
      await _tick();
    }
    if (!condition()) fail(_timeoutEvidence(limit));
  }

  Future<void> _waitAsync(Future<bool> Function() condition) async {
    final watch = Stopwatch()..start();
    while (!await condition() && watch.elapsed < const Duration(seconds: 30)) {
      await _tick();
    }
    if (!await condition()) fail(_timeoutEvidence(watch.elapsed));
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
    _sampleVisibleState();
  }

  void _sampleVisibleState() {
    final mounted = find.byType(VideoPlayer, skipOffstage: false).evaluate();
    peakMountedPlayers = mounted.length > peakMountedPlayers
        ? mounted.length
        : peakMountedPlayers;
    final capacity = videoPlaybackCapacityOf(graph.playback).inUse;
    if (capacity > peakControllerCapacity) peakControllerCapacity = capacity;
    unavailableWasVisible |= find
        .text('Video unavailable')
        .evaluate()
        .isNotEmpty;
    activePlaceholderWasVisible |= _activePlaceholderVisible();
  }

  bool _activePlaceholderVisible() {
    final state = graph.cubit.state;
    if (state is! FeedLoaded) return false;
    final id = state.posts[state.activeIndex].id.value;
    return find
        .byWidgetPredicate(
          (widget) => widget is ColoredBox && widget.key == ValueKey(id),
          skipOffstage: false,
        )
        .evaluate()
        .isNotEmpty;
  }

  String _timeoutEvidence(Duration timeout) =>
      'Cache-pressure journey timed out after $timeout; '
      'state=${graph.cubit.state.runtimeType}, handoffs=$forwardHandoffs, '
      'requests=${origin.requests.length}, active=${origin.activeIncompleteRequestSequences}.';
}
