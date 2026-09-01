part of 'warp_long_session_scenario.dart';

const _longSessionLegs = [
  (direction: _LongSwipeDirection.forward, count: 23),
  (direction: _LongSwipeDirection.backward, count: 3),
];

const _longSessionCancellationWarmup = 3;

final class _WarpLongSessionDriver {
  _WarpLongSessionDriver(this.scenario, this.tester);

  final WarpLongSessionScenario scenario;
  final WidgetTester tester;
  final visited = <String>{};
  late final ProgressiveOriginPreBodyGate cancellationGate;
  late final ProgressiveOriginRequest cancellationRequest;
  late final int cancellationDecisionSequence;
  var handoffs = 0;
  var decodedHandoffs = 0;
  var transportRescues = 0;
  var peakMountedPlayers = 0;
  var peakControllerCapacity = 0;
  var unavailableWasVisible = false;
  var activePlaceholderWasVisible = false;

  WarpFeedProductionGraph get graph => scenario.graph;
  ProgressiveDeviceOrigin get origin => scenario.resources.origin;

  Future<void> run() async {
    _armSwipeCancellation();
    try {
      await _open();
      await _swipeLeg(
        _LongSwipeDirection.forward,
        _longSessionCancellationWarmup,
      );
      await _swipeCancellationBurst();
      for (final leg in _longSessionLegs) {
        await _swipeLeg(leg.direction, leg.count);
      }
      _expectSessionBounded();
      await _teardown();
      _expectQuiescent();
    } finally {
      cancellationGate.release();
    }
  }

  Future<void> _open() async {
    await tester.pumpWidget(MaterialApp(home: WarpFeedSurface(graph: graph)));
    unawaited(graph.cubit.load());
    await _wait(() => _loadedPostCount >= _longSessionPostCount);
    final focus = await _waitForInitialFocus();
    visited.add(focus.videoId.value);
    await _waitForDecodedPlayback(focus);
  }

  Future<PlaybackFocus> _waitForInitialFocus() async {
    PlaybackFocus? focus;
    await _wait(() {
      focus = graph.focus.occurrenceAfter(scenario.events.first.id, 0);
      return focus != null;
    });
    return focus!;
  }

  int get _loadedPostCount {
    final state = graph.cubit.state;
    return state is FeedLoaded ? state.posts.length : 0;
  }

  Future<void> _teardown() async {
    final baseline = await _latestPlanRevision();
    await tester.pumpWidget(const SizedBox.shrink());
    _sampleVisibleState();
    await _waitForStableQuiescence(baseline);
  }
}
