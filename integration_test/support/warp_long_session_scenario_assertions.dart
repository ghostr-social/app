part of 'warp_long_session_scenario.dart';

extension _WarpLongSessionAssertions on _WarpLongSessionDriver {
  void _expectSessionBounded() {
    expect(handoffs, 32);
    expect(
      decodedHandoffs,
      handoffs - _unsettledBurstHandoffs - transportRescues,
    );
    expect(visited, hasLength(_longSessionPostCount));
    expect(peakMountedPlayers, lessThanOrEqualTo(8));
    expect(peakControllerCapacity, lessThanOrEqualTo(8));
    expect(unavailableWasVisible, isFalse);
    expect(activePlaceholderWasVisible, isFalse);
    expect(transportRescues, greaterThanOrEqualTo(1));
    expect(graph.focus.hadTransportRescue, isTrue);
  }

  bool _isQuiescent() {
    final requestsStopped = origin.requests.every(
      (request) =>
          request.outcome == ProgressiveOriginRequestOutcome.completed ||
          request.outcome == ProgressiveOriginRequestOutcome.clientCanceled,
    );
    return _begunAttempts.every((attempt) => attempt.releasedAt != null) &&
        find.byType(VideoPlayer, skipOffstage: false).evaluate().isEmpty &&
        videoPlaybackCapacityOf(graph.playback).isQuiescent &&
        origin.activeIncompleteRequestSequences.isEmpty &&
        requestsStopped;
  }

  void _expectQuiescent() {
    final attempts = _begunAttempts;
    expect(attempts.length, greaterThan(8));
    expect(attempts.every((attempt) => attempt.failedAt == null), isTrue);
    expect(attempts.every((attempt) => attempt.releasedAt != null), isTrue);
    expect(_peakPreparations(attempts), lessThanOrEqualTo(8));
    expect(find.byType(VideoPlayer, skipOffstage: false), findsNothing);
    _expectOriginBounded();
    debugPrint(
      'WARP_LONG_SESSION handoffs=$handoffs visited=${visited.length} '
      'mounted_peak=$peakMountedPlayers attempts=${attempts.length} '
      'controller_peak=$peakControllerCapacity '
      'attempt_peak=${_peakPreparations(attempts)} '
      'origin_peak=${origin.maximumConcurrentResponses} '
      'requests=${origin.requests.length} '
      'rescues=$transportRescues '
      'canceled=${_canceledRequestCount()}',
    );
  }

  List<WarpFeedPlayerStageEvidence> get _begunAttempts =>
      _allAttempts.where((attempt) => attempt.initializingAt != null).toList();

  List<WarpFeedPlayerStageEvidence> get _allAttempts {
    final attempts = <WarpFeedPlayerStageEvidence>{};
    for (final event in scenario.events) {
      final delivery = graph.focus.deliveryForEvent(event.id);
      if (delivery != null) {
        attempts.addAll(graph.playerStages.attemptsFor(delivery));
      }
    }
    return attempts.toList();
  }

  int _peakPreparations(List<WarpFeedPlayerStageEvidence> attempts) {
    var peak = 0;
    for (final candidate in attempts) {
      final at = candidate.initializingAt!;
      final active = attempts.where((attempt) {
        final started = attempt.initializingAt!;
        final terminal = attempt.releasedAt ?? attempt.failedAt;
        return started <= at && (terminal == null || terminal > at);
      }).length;
      if (active > peak) peak = active;
    }
    return peak;
  }
}
