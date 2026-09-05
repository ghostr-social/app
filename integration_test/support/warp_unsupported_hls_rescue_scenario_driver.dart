part of 'warp_unsupported_hls_rescue_scenario.dart';

final class _WarpUnsupportedHlsRescueDriver {
  _WarpUnsupportedHlsRescueDriver(this.runtime, this.tester);

  final WarpUnsupportedHlsRescueRuntime runtime;
  final WidgetTester tester;
  var peakMountedPlayers = 0;
  var peakControllerCapacity = 0;
  var unavailableWasVisible = false;

  WarpFeedProductionGraph get graph => runtime.graph;
  FeedLoaded get feed => graph.cubit.state as FeedLoaded;
  ProgressiveDeviceOrigin get progressive => runtime.resources.origin;

  /// Captured while the feed is loaded: the cleanup stages run after the
  /// surface is unmounted, when the cubit no longer holds a loaded state.
  late final PlaybackDeliveryId failedDeliveryId;
  late final PlaybackDeliveryId alternateDeliveryId;

  Future<void> run() async {
    await _mount();
    await _navigateToAlternate();
    final evidence = await _waitForDecodedAlternate();
    _expectLiveContract(evidence);
    await _unmount();
    await _waitForQuiescence();
    _expectBoundedCleanup(evidence);
    _report(evidence);
  }

  Future<void> _mount() async {
    await tester.pumpWidget(MaterialApp(home: WarpFeedSurface(graph: graph)));
    unawaited(graph.cubit.load());
    await _wait(_isLoaded);
    failedDeliveryId = feed.posts[0].media.playbackDeliveryId!;
    alternateDeliveryId = feed.posts[1].media.playbackDeliveryId!;
  }

  bool _isLoaded() {
    final state = graph.cubit.state;
    return state is FeedLoaded && state.posts.length == 2;
  }

  Future<void> _navigateToAlternate() async {
    await _wait(
      () =>
          _failure() != null &&
          find.text('Video unavailable').evaluate().isNotEmpty,
    );
    expect(feed.activeIndex, 0);
    expect(graph.focus.hadTransportRescue, isFalse);
    final page = find.byType(PageView);
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, -tester.getSize(page).height * 0.3));
    await tester.pump(const Duration(milliseconds: 16));
    await gesture.up();
  }

  Future<void> _unmount() async {
    await tester.pumpWidget(const SizedBox.shrink());
  }
}
