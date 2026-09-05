part of 'warp_cache_pressure_scenario.dart';

final class _WarpCachePressureDriver {
  _WarpCachePressureDriver(this.scenario, this.tester);

  final WarpCachePressureScenario scenario;
  final WidgetTester tester;
  late PlaybackFocus coldFocus;
  late PlaybackDeliveryId coldDelivery;
  late int coldPlayerGeneration;
  var forwardHandoffs = 0;
  var coldBytesBeforeReturn = 0;
  var peakMountedPlayers = 0;
  var peakControllerCapacity = 0;
  var unavailableWasVisible = false;
  var activePlaceholderWasVisible = false;

  WarpLongSessionScenario get session => scenario.session;
  WarpFeedProductionGraph get graph => session.graph;
  ProgressiveDeviceOrigin get origin => session.resources.origin;

  Future<void> run() async {
    await _open();
    await _driveUntilColdEvicted();
    coldBytesBeforeReturn = origin.bytesServed('long-00');
    final returned = await _returnToCold();
    await _expectDecodedAndAdvancing(returned);
    await _expectPressureContract();
    await _teardown();
    _expectQuiescent();
  }

  Future<void> _open() async {
    await tester.pumpWidget(MaterialApp(home: WarpFeedSurface(graph: graph)));
    unawaited(graph.cubit.load());
    await _wait(() => _loadedPostCount == session.events.length);
    coldFocus = await _waitForFocus(session.events.first.id, 0);
    coldDelivery = graph.focus.deliveryForEvent(session.events.first.id)!;
    await _waitForDecodedPlayback(coldFocus);
    coldPlayerGeneration = graph.telemetry.probe
        .sessionFor(coldFocus)!
        .generation;
    await _waitForColdCoverage();
  }

  int get _loadedPostCount {
    final state = graph.cubit.state;
    return state is FeedLoaded ? state.posts.length : 0;
  }
}
