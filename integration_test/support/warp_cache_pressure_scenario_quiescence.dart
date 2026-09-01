part of 'warp_cache_pressure_scenario.dart';

extension _WarpCachePressureQuiescence on _WarpCachePressureDriver {
  Future<void> _teardown() async {
    final baseline =
        (await graph.evidence.page()).planPage.latestRetainedRevision;
    await tester.pumpWidget(const SizedBox.shrink());
    final limit =
        playbackControllerTeardownTimeout + const Duration(seconds: 2);
    await _wait(_isQuiescent, timeout: limit);
    await _waitForNativeRelease(baseline, limit);
    await _pumpFor(const Duration(seconds: 1));
  }

  bool _isQuiescent() {
    final stopped = origin.requests.every(
      (request) =>
          request.outcome == ProgressiveOriginRequestOutcome.completed ||
          request.outcome == ProgressiveOriginRequestOutcome.clientCanceled,
    );
    return _attempts.every((attempt) => attempt.isTerminal) &&
        videoPlaybackCapacityOf(graph.playback).isQuiescent &&
        origin.activeIncompleteRequestSequences.isEmpty &&
        find.byType(VideoPlayer, skipOffstage: false).evaluate().isEmpty &&
        stopped;
  }

  Future<void> _waitForNativeRelease(int baseline, Duration limit) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < limit) {
      final page = await graph.evidence.page(afterRevision: baseline);
      final released = page.planPage.records.any(
        (plan) =>
            plan.revision > baseline && plan.playerVerifiedPostIds.isEmpty,
      );
      if (released) return;
      await _tick();
    }
    fail('Native preparations did not release after revision $baseline.');
  }

  void _expectQuiescent() {
    expect(_isQuiescent(), isTrue);
    expect(_attempts, isNotEmpty);
    expect(_attempts.every((attempt) => attempt.isTerminal), isTrue);
    debugPrint(
      'WARP_CACHE_PRESSURE handoffs=$forwardHandoffs '
      'controller_peak=$peakControllerCapacity origin_peak='
      '${origin.maximumConcurrentResponses} requests=${origin.requests.length}',
    );
  }

  List<WarpFeedPlayerStageEvidence> get _attempts {
    final result = <WarpFeedPlayerStageEvidence>{};
    for (final event in session.events) {
      final delivery = graph.focus.deliveryForEvent(event.id);
      if (delivery != null) {
        result.addAll(graph.playerStages.attemptsFor(delivery));
      }
    }
    return result.toList();
  }
}
