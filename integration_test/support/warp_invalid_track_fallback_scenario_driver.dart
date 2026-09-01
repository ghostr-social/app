part of 'warp_invalid_track_fallback_scenario.dart';

final class _WarpInvalidTrackFallbackDriver {
  _WarpInvalidTrackFallbackDriver(this.scenario, this.tester);

  final WarpInvalidTrackFallbackScenario scenario;
  final WidgetTester tester;
  var peakMountedPlayers = 0;
  var peakControllerCapacity = 0;
  var unavailableWasVisible = false;
  late final PlaybackDeliveryId testedDeliveryId;

  WarpFeedProductionGraph get graph => scenario.graph;
  ProgressiveDeviceOrigin get origin => scenario.resources.origin;

  Future<void> run() async {
    await _open();
    final evidence = await _waitForFallback();
    final advance = await _waitForAdvancement(evidence.focus);
    _expectLiveContract(evidence, advance);
    await _teardown();
    _expectQuiescent(evidence);
    _report(evidence, advance);
  }

  Future<void> _open() async {
    await tester.pumpWidget(
      MaterialApp(
        home: WarpFeedSurface(graph: graph, playback: scenario.playback),
      ),
    );
    unawaited(graph.cubit.load());
    await _wait(_isLoaded);
    testedDeliveryId = _loadedDeliveryId;
  }

  bool _isLoaded() {
    final state = graph.cubit.state;
    return state is FeedLoaded &&
        state.posts.length == 1 &&
        state.activeIndex == 0;
  }

  Future<void> _teardown() async {
    await tester.pumpWidget(const SizedBox.shrink());
    await _wait(_isQuiescent, timeout: const Duration(seconds: 15));
    await _pumpFor(const Duration(seconds: 1));
    expect(_isQuiescent(), isTrue);
  }

  bool _isQuiescent() {
    final stages = graph.playerStages.attemptsFor(testedDeliveryId);
    return stages.isNotEmpty &&
        stages.every((stage) => stage.isTerminal) &&
        videoPlaybackCapacityOf(scenario.playback).isQuiescent &&
        origin.activeIncompleteRequestSequences.isEmpty &&
        find.byType(VideoPlayer, skipOffstage: false).evaluate().isEmpty;
  }

  PlaybackDeliveryId get _loadedDeliveryId {
    final state = graph.cubit.state as FeedLoaded;
    return state.posts.single.media.playbackDeliveryId!;
  }
}
