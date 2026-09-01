part of 'warp_stale_validator_rotation_scenario.dart';

final class _WarpValidatorRotationDriver {
  _WarpValidatorRotationDriver(this.scenario, this.tester);

  final _WarpValidatorRotationScenario scenario;
  final WidgetTester tester;
  var peakMountedPlayers = 0;
  var peakControllerCapacity = 0;
  var unavailableWasVisible = false;

  WarpFeedProductionGraph get graph => scenario.graph;
  WarpValidatorRotationFixture get fixture => scenario.fixture;

  Future<void> run() async {
    await _open();
    final first = await _waitForFocus(0);
    await _wait(() => fixture.hasHeldFirstRequest);
    final stable = await _leaveHeldGeneration(first);
    final replacement = await _returnToReplacement(stable);
    final advance = await _waitForAdvancement(replacement);
    final bytes = await _readReplacementBytes();
    _expectLiveContract(replacement, advance, bytes);
    await _teardown();
    _expectQuiescent();
    _report(replacement, advance, bytes);
  }

  Future<void> _open() async {
    await tester.pumpWidget(MaterialApp(home: WarpFeedSurface(graph: graph)));
    unawaited(graph.cubit.load());
    await _wait(() {
      final state = graph.cubit.state;
      return state is FeedLoaded && state.posts.length == 2;
    });
  }

  Future<PlaybackFocus> _leaveHeldGeneration(PlaybackFocus first) async {
    final cursor = graph.focus.occurrences.last.sequence;
    await _swipe(-1);
    final stable = await _waitForFocus(1, afterSequence: cursor);
    await _waitForDecoded(stable);
    expect(graph.telemetry.probe.sessionFor(first), isNull);
    fixture.rotate();
    fixture.releaseFirstGeneration();
    await _wait(_firstBodiesRetired);
    return stable;
  }

  Future<PlaybackFocus> _returnToReplacement(PlaybackFocus stable) async {
    final cursor = graph.focus.occurrences.last.sequence;
    await _swipe(1);
    final replacement = await _waitForFocus(0, afterSequence: cursor);
    await _waitForDecoded(replacement);
    expect(graph.telemetry.probe.sessionFor(stable), isNull);
    return replacement;
  }
}
