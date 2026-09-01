part of 'warp_long_session_scenario.dart';

extension _WarpLongSessionWait on _WarpLongSessionDriver {
  Future<void> _waitForDecodedPlayback(PlaybackFocus focus) async {
    const timeout = Duration(seconds: 30);
    final watch = Stopwatch()..start();
    while (!_hasDecodedPlayback(focus) && watch.elapsed < timeout) {
      await _tick();
    }
    if (!_hasDecodedPlayback(focus)) {
      await _failDecodedPlayback(focus, timeout);
    }
    expect(find.text('Video unavailable'), findsNothing);
    _expectNoActivePlaceholder(focus);
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

  Future<void> _wait(
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 30),
    String? awaiting,
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await _tick();
    }
    if (!condition()) fail(_timeoutEvidence(timeout, awaiting));
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
    final mounted = find
        .byType(VideoPlayer, skipOffstage: false)
        .evaluate()
        .length;
    if (mounted > peakMountedPlayers) peakMountedPlayers = mounted;
    final capacity = videoPlaybackCapacityOf(graph.playback).inUse;
    if (capacity > peakControllerCapacity) peakControllerCapacity = capacity;
    unavailableWasVisible |= find
        .text('Video unavailable')
        .evaluate()
        .isNotEmpty;
    final state = graph.cubit.state;
    if (state is FeedLoaded) {
      activePlaceholderWasVisible |= _hasPlaceholder(
        state.posts[state.activeIndex].id.value,
      );
    }
  }

  void _expectNoActivePlaceholder(PlaybackFocus focus) {
    expect(_hasPlaceholder(focus.videoId.value), isFalse);
  }

  bool _hasPlaceholder(String videoId) => find
      .byWidgetPredicate(
        (widget) => widget is ColoredBox && widget.key == ValueKey(videoId),
        skipOffstage: false,
      )
      .evaluate()
      .isNotEmpty;

  String _timeoutEvidence(Duration timeout, String? awaiting) {
    final loaded = graph.cubit.state;
    final active = loaded is FeedLoaded
        ? '${loaded.activeIndex}:${loaded.posts[loaded.activeIndex].id.value}'
        : 'none';
    return 'Long WARP session timed out after $timeout; '
        'awaiting=${awaiting ?? 'condition'}, activePage=$active, '
        'state=${graph.cubit.state.runtimeType}, posts=$_loadedPostCount, '
        'focuses=${graph.focus.occurrences.length}, handoffs=$handoffs, '
        'players=$peakMountedPlayers, requests=${origin.requests.length}, '
        'active=${origin.activeIncompleteRequestSequences}.';
  }
}
