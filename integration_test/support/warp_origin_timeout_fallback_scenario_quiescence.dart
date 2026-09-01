part of 'warp_origin_timeout_fallback_scenario.dart';

extension _OriginTimeoutFallbackQuiescence on _OriginTimeoutFallbackScenario {
  Future<void> expectQuiescentAfterUnmount(WidgetTester tester) async {
    expect(_playerAttempts, isNotEmpty);
    await tester.pumpWidget(const SizedBox.shrink());
    final limit =
        playbackControllerTeardownTimeout + const Duration(seconds: 2);
    final watch = Stopwatch()..start();
    while (!_isQuiescent && watch.elapsed < limit) {
      await journey.pumpFor(tester, const Duration(milliseconds: 50));
    }
    expect(_isQuiescent, isTrue, reason: _quiescenceEvidence(limit));
    expect(
      videoPlaybackCapacityOf(journey.playback),
      emptyVideoPlaybackCapacitySnapshot,
    );
  }

  bool get _isQuiescent =>
      _playerAttempts.every((attempt) => attempt.isTerminal) &&
      videoPlaybackCapacityOf(journey.playback).isQuiescent &&
      journey.resources.origin.activeIncompleteRequestSequences.isEmpty &&
      !journey.resources.origin.headsRemainBlocked;

  List<WarpFeedPlayerStageEvidence> get _playerAttempts {
    final attempts = <WarpFeedPlayerStageEvidence>{};
    for (final event in journey.events) {
      final delivery = journey.focus.deliveryForEvent(event.id);
      if (delivery != null) {
        attempts.addAll(journey.playerStages.attemptsFor(delivery));
      }
    }
    return attempts.toList();
  }

  String _quiescenceEvidence(Duration limit) {
    return 'Origin-timeout teardown exceeded $limit; '
        'attempts=${_playerAttempts.length}, '
        'terminal=${_playerAttempts.where((item) => item.isTerminal).length}, '
        'capacity=${videoPlaybackCapacityOf(journey.playback)}, '
        'active=${journey.resources.origin.activeIncompleteRequestSequences}, '
        'heads=${journey.resources.origin.headsRemainBlocked}.';
  }
}
