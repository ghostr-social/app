part of 'warp_origin_timeout_fallback_scenario.dart';

const _minimumFailoverDelay = Duration(seconds: 10);
const _maximumFailoverDelay = Duration(seconds: 20);

extension _OriginTimeoutFallbackAssertions on _OriginTimeoutFallbackScenario {
  void expectBoundedFailover(_OriginTimeoutEvidence evidence) {
    expect(primaryGate.isReached, isTrue);
    expect(primaryGate.isReleased, isFalse);
    expect(primaryGate.timedOut, isFalse);
    expect(evidence.primary.servedBytes, 0);
    expect(evidence.primary.firstByteAt, isNull);
    expect(evidence.failoverDelay, greaterThanOrEqualTo(_minimumFailoverDelay));
    expect(evidence.failoverDelay, lessThanOrEqualTo(_maximumFailoverDelay));
    expect(evidence.fallbackBytes, journey.resources.origin.objectLength);
    expect(evidence.stage.firstFrameAt, isNotNull);
    expect(evidence.stage.isTerminal, isFalse);
    _expectOnePrimaryGet();
    _expectBoundedExactFallback();
  }

  Future<PlaybackFocus> swipeToIntendedNext(WidgetTester tester) async {
    final cursor = journey.focusCursor;
    await journey.swipeUp(tester);
    final focus = await journey.waitForPublishedFocus(
      tester,
      1,
      afterSequence: cursor,
      cause: FeedFocusCause.userNavigation,
    );
    await journey.waitForCaption(tester, 1);
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
    return focus;
  }

  Future<void> expectDecodedNext(
    WidgetTester tester,
    PlaybackFocus focus,
    _OriginTimeoutEvidence evidence,
  ) async {
    expect(primaryGate.isReleased, isFalse);
    final attempts = journey.playerStages.attemptsFor(nextId);
    expect(
      warpOriginTimeoutDecodedStage(attempts, evidence.stage.authority),
      same(evidence.stage),
    );
    expect(attempts.where((attempt) => !attempt.isTerminal), [
      same(evidence.stage),
    ]);
    expect(find.text('Video unavailable'), findsNothing);
    final before = journey.telemetry.probe.latestPositionFor(focus)!;
    await journey.pumpFor(tester, const Duration(seconds: 1));
    expect(
      journey.telemetry.probe.latestPositionFor(focus),
      greaterThan(before),
    );
    _expectOnePrimaryGet();
    debugPrint(
      'WARP_ORIGIN_TIMEOUT failover_ms='
      '${evidence.failoverDelay.inMilliseconds} primary_gets=1 '
      'fallback_gets=${evidence.fallback.length} '
      'fallback_bytes=${evidence.fallbackBytes}',
    );
  }

  void _expectOnePrimaryGet() {
    final primary = journey.resources.origin
        .requestsFor('next')
        .where((request) => request.method == 'GET');
    expect(primary, hasLength(1));
  }

  void _expectBoundedExactFallback() {
    final fallback = journey.resources.origin
        .requestsFor('next-rescue')
        .where((request) => request.method == 'GET')
        .toList();
    expect(
      warpOriginTimeoutHasBoundedExactFallback(
        fallback,
        objectLength: journey.resources.origin.objectLength,
      ),
      isTrue,
    );
  }
}
